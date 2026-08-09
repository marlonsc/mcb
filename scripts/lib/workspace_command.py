"""Generic dynamic command framework for workspace Make wrappers.

This file is distributed by ~/.ai-hub to workspaces catalogued in CRG.
When copied into <workspace>/scripts/lib/, it imports workspace-local paths
from <workspace>/scripts/lib/workspace.py.
"""

from __future__ import annotations

import os
import runpy
import subprocess
import sys
from collections.abc import Callable, Iterable, Mapping, Sequence
from dataclasses import dataclass
from datetime import date, datetime, time
from pathlib import Path
from typing import NoReturn

import toml

from lib.workspace import (
    LOCAL_PYTHON,
    ROOT,
    SCRIPTS,
    SUBMODULE_SCRIPT_ROOTS,
    local_python_cmd,
    runtime_env,
)

TomlValue = str | int | float | bool | list["TomlValue"] | dict[str, "TomlValue"]
TomlTable = dict[str, TomlValue]
RawTomlValue = (
    str
    | int
    | float
    | bool
    | list["RawTomlValue"]
    | dict[str, "RawTomlValue"]
    | date
    | datetime
    | time
)

HEADER_START = "/// workspace-command"
HEADER_END = "///"
COMMAND_SUFFIXES = frozenset({".sh", ".py"})
IGNORED_DIRS = frozenset({"__pycache__", "hooks", "legado", "lib"})
MUTATION_REQUIRED_PARAMS = frozenset({"APPLY"})
INCIDENT_MUTATION_REQUIRED_PARAMS = frozenset({
    "APPLY",
    "EMERGENCY",
    "BREAKING_GLASS_BEAD",
})


@dataclass(frozen=True, slots=True)
class Param:
    name: str
    help: str
    required: bool = False
    default: str = ""
    choices: tuple[str, ...] = ()


@dataclass(frozen=True, slots=True)
class Command:
    verb: str
    what: str
    domain: str
    summary: str
    description: str
    example: str
    path: Path
    mutates: bool
    aliases: tuple[str, ...]
    params: tuple[Param, ...]
    rules: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class AliasTarget:
    verb: str
    what: str


class RegistryError(Exception):
    """Raised when command metadata or invocation is invalid."""


class Registry:
    """In-memory view discovered from script headers; never a static catalog."""

    def __init__(self) -> None:
        self._commands: dict[str, dict[str, Command]] = {}
        self._aliases: dict[str, AliasTarget] = {}

    def add(self, command: Command) -> None:
        by_what = self._commands.setdefault(command.verb, {})
        if command.what in by_what:
            msg = f"comando duplicado: {command.verb} WHAT={command.what}"
            raise RegistryError(msg)
        by_what[command.what] = command
        for alias in command.aliases:
            alias_name, target_what = parse_alias_spec(alias, command)
            target = AliasTarget(command.verb, target_what)
            previous = self._aliases.get(alias_name)
            if previous and previous != target:
                msg = (
                    f"alias duplicado: {alias_name} aponta para {previous.verb} WHAT={previous.what} "
                    f"e {target.verb} WHAT={target.what}"
                )
                raise RegistryError(msg)
            self._aliases[alias_name] = target

    def validate(self) -> None:
        if not self._commands:
            msg = "nenhum comando promovido encontrado em scripts/<verbo>/<WHAT>"
            raise RegistryError(msg)
        for verb, commands in sorted(self._commands.items()):
            if "all" not in commands:
                msg = f"verbo '{verb}' sem WHAT=all"
                raise RegistryError(msg)
            domains = {command.domain for command in commands.values()}
            if len(domains) != 1:
                valid = ", ".join(sorted(domains))
                msg = f"verbo '{verb}' declara mais de um domain: {valid}"
                raise RegistryError(msg)
            for command in commands.values():
                if command.what != "all" and command.aliases:
                    msg = f"{command.path}: aliases devem ser declarados apenas em WHAT=all"
                    raise RegistryError(msg)
                validate_command_contract(command)
            validate_all_choices(verb, commands)
        for alias, target in self._aliases.items():
            if alias in self._commands:
                msg = f"alias '{alias}' colide com verbo promovido"
                raise RegistryError(msg)
            if target.verb not in self._commands:
                msg = f"alias '{alias}' aponta para verbo desconhecido {target.verb}"
                raise RegistryError(msg)
            if target.what not in self._commands[target.verb]:
                msg = f"alias '{alias}' aponta para {target.verb} WHAT={target.what}, mas a acao nao existe"
                raise RegistryError(msg)

    def verbs(self) -> list[str]:
        return sorted(self._commands)

    def resolve_verb(self, verb: str) -> str:
        target = self._aliases.get(verb)
        resolved = target.verb if target is not None else verb
        if resolved not in self._commands:
            msg = f"verbo '{verb}' desconhecido"
            raise RegistryError(msg)
        return resolved

    def alias_target(self, verb: str) -> AliasTarget | None:
        return self._aliases.get(verb)

    def commands(self, verb: str) -> Mapping[str, Command]:
        return self._commands[self.resolve_verb(verb)]

    def command(self, verb: str, what: str) -> Command:
        commands = self.commands(verb)
        if what not in commands:
            valid = " ".join(sorted(commands))
            msg = f"WHAT='{what}' invalido para {verb}. Validos: {valid}"
            raise RegistryError(msg)
        return commands[what]

    def aliases_for(self, verb: str) -> list[str]:
        return sorted(
            alias for alias, target in self._aliases.items() if target.verb == verb
        )


def main(argv: Sequence[str] | None = None) -> int:
    args = tuple(sys.argv[1:] if argv is None else argv)
    try:
        ensure_local_python()
        registry = discover()
        if args and args[0] == "--validate":
            return 0
        if not args or args[0] in {"help", "--help", "-h"}:
            os.environ.get("WHAT", "").strip()
            return 0
        return dispatch(registry, args[0])
    except RegistryError:
        return 2


def dispatch(registry: Registry, requested_verb: str) -> int:
    alias_target = registry.alias_target(requested_verb)
    verb = registry.resolve_verb(requested_verb)
    requested_what = os.environ.get("WHAT", "").strip()
    default_what = alias_target.what if alias_target is not None else "all"
    what = requested_what or default_what
    if what in {"all", "help"}:
        return 0
    command = registry.command(verb, what)
    if env_enabled("HELP") or env_enabled("OPTIONS"):
        return 0
    is_dry_run = command.mutates and os.environ.get("APPLY", "N") != "Y"
    validate_invocation(command, require_required=not is_dry_run)
    if is_dry_run:
        return 0
    return run(command)


def run(command: Command) -> int:
    env = command_env(command)
    cwd = command_cwd(command)
    if command.path.suffix == ".py":
        return run_python(command, env, cwd)
    return subprocess.run(
        ["bash", str(command.path)], cwd=cwd, env=env, check=False
    ).returncode


def run_python(command: Command, env: Mapping[str, str], cwd: Path) -> int:
    previous_env = os.environ.copy()
    previous_argv = sys.argv[:]
    previous_cwd = Path.cwd()
    previous_path = sys.path[:]
    try:
        os.environ.clear()
        os.environ.update(env)
        sys.argv = [str(command.path)]
        sys.path.insert(0, str(cwd / "scripts"))
        sys.path.insert(1, str(cwd))
        os.chdir(cwd)
        try:
            runpy.run_path(str(command.path), run_name="__main__")
        except SystemExit as exc:
            code = exc.code
            if code is None:
                return 0
            if isinstance(code, int):
                return code
            return 1
        return 0
    finally:
        os.chdir(previous_cwd)
        sys.argv = previous_argv
        sys.path = previous_path
        os.environ.clear()
        os.environ.update(previous_env)


def command_env(command: Command) -> dict[str, str]:
    """Return the canonical environment for a promoted command."""
    env: dict[str, str] = runtime_env(os.environ.copy())
    env["WHAT"] = command.what
    env["WORKSPACE_COMMAND_DISPATCHED"] = "Y"
    env["WORKSPACE_COMMAND_VERB"] = command.verb
    env["WORKSPACE_COMMAND_WHAT"] = command.what
    env["WORKSPACE_COMMAND_DOMAIN"] = command.domain
    env["WORKSPACE_COMMAND_PATH"] = str(command.path.resolve())
    submodule_root = submodule_root_for(command.path)
    if submodule_root is not None:
        env["WORKSPACE_SUBMODULE_ROOT"] = str(submodule_root.resolve())
    env.pop("PYTHONPATH", None)
    return env


def command_cwd(command: Command) -> Path:
    """Return the working directory for executing a promoted command."""
    submodule_root = submodule_root_for(command.path)
    return submodule_root if submodule_root is not None else ROOT


def submodule_root_for(command_path: Path) -> Path | None:
    """Return the submodule root if the command lives inside one."""
    for scripts_root in SUBMODULE_SCRIPT_ROOTS:
        submodule_root = Path(scripts_root).parent
        try:
            command_path.resolve().relative_to(submodule_root.resolve())
        except ValueError:
            continue
        return submodule_root
    return None


def local_python() -> str:
    """Return the repository-local Python command after validating the active interpreter."""
    ensure_local_python()
    return str(local_python_cmd())


def ensure_local_python() -> None:
    """Fail loud unless the command framework is running on the expected Python.

    In a workspace with a local .venv the expected interpreter is .venv/bin/python.
    In standalone workspaces without a local .venv, the active interpreter is accepted
    so the same dispatcher code works when the workspace is used as its own repo.
    """
    expected = local_python_cmd()
    if expected == LOCAL_PYTHON and not LOCAL_PYTHON.is_file():
        msg = f"Python local ausente: {LOCAL_PYTHON}; crie/sincronize .venv antes de usar make"
        raise RegistryError(msg)
    if Path(sys.executable).resolve() != expected.resolve():
        msg = f"Python ativo nao e o esperado: {expected}; use make com PATH da .venv"
        raise RegistryError(msg)


def discover() -> Registry:
    registry = Registry()
    script_roots = _script_roots()
    if not any(root.exists() for root in script_roots):
        msg = "nenhum diretorio scripts encontrado"
        raise RegistryError(msg)
    for scripts_root in script_roots:
        if not scripts_root.exists():
            continue
        for path in sorted(scripts_root.iterdir()):
            if path.name == "__pycache__":
                continue
            if path.is_file():
                continue
            if not path.is_dir():
                msg = f"{path}: entrada invalida em {scripts_root}"
                raise RegistryError(msg)
            if path.name in IGNORED_DIRS:
                continue
            _discover_verb_dir(registry, path)
    registry.validate()
    return registry


def _script_roots() -> list[Path]:
    return [SCRIPTS, *sorted(SUBMODULE_SCRIPT_ROOTS, key=lambda p: str(p))]


def _discover_verb_dir(registry: Registry, verb_dir: Path) -> None:
    if not _has_command_files(verb_dir):
        return
    for path in sorted(verb_dir.iterdir()):
        if path.name == "__pycache__":
            continue
        if path.is_dir():
            msg = f"{path}: diretorio aninhado nao e comando publico"
            raise RegistryError(msg)
        if path.suffix not in COMMAND_SUFFIXES:
            msg = f"{path}: arquivo publico deve ser .sh ou .py"
            raise RegistryError(msg)
        registry.add(load_command(path, verb_dir.name))


def _has_command_files(verb_dir: Path) -> bool:
    return any(
        path.is_file() and path.suffix in COMMAND_SUFFIXES and has_header(path)
        for path in verb_dir.iterdir()
    )


def has_header(path: Path) -> bool:
    try:
        header_data(path)
    except RegistryError:
        return False
    return True


def load_command(path: Path, expected_verb: str) -> Command:
    data = header_data(path)
    verb = require_string(data, "verb", path)
    what = require_string(data, "what", path)
    if verb != expected_verb:
        msg = f"{path}: header verb={verb} diverge do diretorio {expected_verb}"
        raise RegistryError(msg)
    if what != path.stem:
        msg = f"{path}: header what={what} diverge do arquivo {path.stem}"
        raise RegistryError(msg)
    return Command(
        verb=verb,
        what=what,
        domain=require_string(data, "domain", path),
        summary=require_string(data, "summary", path),
        description=require_string(data, "description", path),
        example=require_string(data, "example", path),
        path=path,
        mutates=require_bool(data, "mutates", path),
        aliases=parse_aliases(data.get("aliases"), path),
        params=parse_params(data.get("params"), path),
        rules=parse_string_list(data.get("rules"), "rules", path),
    )


def header_data(path: Path) -> TomlTable:
    lines = path.read_text(encoding="utf-8").splitlines()[:160]
    in_header = False
    payload: list[str] = []
    for raw in lines:
        stripped = raw.strip()
        content = stripped[1:].strip() if stripped.startswith("#") else stripped
        if content == HEADER_START:
            in_header = True
            continue
        if in_header and content == HEADER_END:
            break
        if in_header:
            payload.append(content)
    if not payload:
        msg = f"{path}: sem header workspace-command"
        raise RegistryError(msg)
    try:
        return normalize_toml_table(toml.loads("\n".join(payload)), path)
    except toml.TomlDecodeError as exc:
        msg = f"{path}: header TOML invalido: {exc}"
        raise RegistryError(msg) from exc


def normalize_toml_table(data: RawTomlValue, path: Path) -> TomlTable:
    if not isinstance(data, dict):
        msg = f"{path}: header TOML deve ser uma tabela"
        raise RegistryError(msg)
    table: TomlTable = {}
    for key, value in data.items():
        table[key] = normalize_toml_value(value, path)
    return table


def normalize_toml_value(value: RawTomlValue, path: Path) -> TomlValue:
    if isinstance(value, str | bool | int | float):
        return value
    if isinstance(value, list):
        list_result: list[TomlValue] = [
            normalize_toml_value(item, path) for item in value
        ]
        return list_result
    if isinstance(value, dict):
        table_result: TomlTable = {}
        for key, item in value.items():
            table_result[key] = normalize_toml_value(item, path)
        return table_result
    msg = f"{path}: header TOML nao aceita date/time"
    raise RegistryError(msg)


def require_string(data: Mapping[str, TomlValue], key: str, path: Path) -> str:
    value = data.get(key)
    if not isinstance(value, str) or not value.strip():
        msg = f"{path}: campo obrigatorio ausente: {key}"
        raise RegistryError(msg)
    return value.strip()


def require_bool(data: Mapping[str, TomlValue], key: str, path: Path) -> bool:
    value = data.get(key)
    if not isinstance(value, bool):
        msg = f"{path}: campo booleano obrigatorio ausente: {key}"
        raise RegistryError(msg)
    return value


def parse_aliases(value: TomlValue | None, path: Path) -> tuple[str, ...]:
    return parse_string_list(value, "aliases", path)


def parse_alias_spec(alias: str, command: Command) -> tuple[str, str]:
    alias_name, separator, target_what = alias.partition("=")
    alias_name = alias_name.strip()
    target_what = target_what.strip() if separator else command.what
    if not alias_name or not target_what:
        msg = f"{command.path}: alias invalido {alias!r}; use alias ou alias=WHAT"
        raise RegistryError(msg)
    return alias_name, target_what


def parse_string_list(
    value: TomlValue | None, field: str, path: Path
) -> tuple[str, ...]:
    if value is None:
        return ()
    if not isinstance(value, list):
        msg = f"{path}: {field} deve ser lista de strings"
        raise RegistryError(msg)
    values: list[str] = []
    for item in value:
        if not isinstance(item, str) or not item.strip():
            msg = f"{path}: {field} invalido"
            raise RegistryError(msg)
        values.append(item.strip())
    return tuple(values)


def parse_params(value: TomlValue | None, path: Path) -> tuple[Param, ...]:
    if value is None:
        return ()
    if not isinstance(value, list):
        msg = f"{path}: params deve ser lista de objetos"
        raise RegistryError(msg)
    params: list[Param] = []
    for item in value:
        if not isinstance(item, dict):
            msg = f"{path}: params deve conter objetos TOML"
            raise RegistryError(msg)
        params.append(parse_param(item, path))
    return tuple(params)


def parse_param(data: Mapping[str, TomlValue], path: Path) -> Param:
    name = require_string(data, "name", path)
    help_text = require_string(data, "help", path)
    required_raw = data.get("required", False)
    default_raw = data.get("default", "")
    if not isinstance(required_raw, bool):
        msg = f"{path}: params.required deve ser booleano"
        raise RegistryError(msg)
    if not isinstance(default_raw, str):
        msg = f"{path}: params.default deve ser string"
        raise RegistryError(msg)
    return Param(
        name=name,
        help=help_text,
        required=required_raw,
        default=default_raw,
        choices=parse_string_list(data.get("choices"), "params.choices", path),
    )


def render_requested_help(registry: Registry, requested: str) -> str:
    if not requested:
        return render_global_help(registry)
    if "/" in requested:
        verb, what = requested.split("/", 1)
        return render_command_help(registry, verb, what)
    target = registry.alias_target(requested)
    if target is not None and target.what != "all":
        return render_command_help(registry, requested, target.what)
    return render_verb_help(registry, requested)


def render_global_help(registry: Registry) -> str:
    lines = ["workspace - make <verbo> WHAT=<acao> [PARAM=value ...]", ""]
    for verb in registry.verbs():
        command = registry.command(verb, "all")
        aliases = registry.aliases_for(verb)
        suffix = f" (alias: {', '.join(aliases)})" if aliases else ""
        lines.append(f"  {verb:14} [{command.domain:12}] {command.summary}{suffix}")
    lines.extend([
        "",
        "make <verbo> mostra o help do verbo e todos os WHAT.",
        "make help WHAT=<verbo> mostra o mesmo help.",
        "make help WHAT=<verbo>/<acao> ou make <verbo> WHAT=<acao> OPTIONS=Y mostra uma acao.",
        "Comandos mutadores exigem APPLY=Y.",
        "Novos comandos vivem em scripts/<verbo>/<WHAT>.sh|py com header workspace-command.",
    ])
    return "\n".join(lines)


def render_verb_help(registry: Registry, requested_verb: str) -> str:
    verb = registry.resolve_verb(requested_verb)
    aliases = registry.aliases_for(verb)
    alias_suffix = f" (alias: {', '.join(aliases)})" if aliases else ""
    lines = [
        f"make {requested_verb} WHAT=<WHAT>{alias_suffix}",
        "",
        "WHAT disponiveis:",
    ]
    commands = registry.commands(verb)
    for what, command in sorted(commands.items()):
        marker = " [mutates]" if command.mutates else ""
        lines.append(f"  {what:20} [{command.domain:12}] {command.summary}{marker}")
    command_params = [
        (what, command) for what, command in sorted(commands.items()) if command.params
    ]
    if command_params:
        lines.extend(["", "Opcoes por WHAT:"])
        for what, command in command_params:
            lines.append(f"  {what:20} {format_params_inline(command.params)}")
        lines.extend([
            "",
            "Detalhe de uma acao:",
            f"  make help WHAT={requested_verb}/<WHAT>",
            f"  make {requested_verb} WHAT=<WHAT> OPTIONS=Y",
        ])
    rules = sorted({rule for command in commands.values() for rule in command.rules})
    if rules:
        lines.extend(["", "Regras:"])
        lines.extend(f"  - {rule}" for rule in rules)
    examples = sorted({
        example_for(command, requested_verb) for command in commands.values()
    })
    if examples:
        lines.extend(["", "Exemplos:"])
        lines.extend(f"  {example}" for example in examples)
    return "\n".join(lines)


def render_command_help(registry: Registry, requested_verb: str, what: str) -> str:
    command = registry.command(requested_verb, what)
    lines = [
        f"make {requested_verb} WHAT={what}",
        "",
        f"Dominio: {command.domain}",
        f"Muta: {'sim' if command.mutates else 'nao'}",
    ]
    if command.mutates:
        lines.append("Dry-run: sem APPLY=Y, o dispatcher nao executa a acao.")
    lines.extend(["", command.summary, command.description])
    if command.params:
        lines.extend(["", "Parametros:"])
        for param in command.params:
            required = " obrigatorio" if param.required else ""
            default = f" default={param.default}" if param.default else ""
            choices = f" choices={','.join(param.choices)}" if param.choices else ""
            lines.append(f"  {param.name:24} {param.help}{required}{default}{choices}")
    if command.rules:
        lines.extend(["", "Regras:"])
        lines.extend(f"  - {rule}" for rule in command.rules)
    lines.extend(["", "Exemplo:", f"  {example_for(command, requested_verb)}"])
    return "\n".join(lines)


def render_dry_run(command: Command, requested_verb: str, what: str) -> str:
    lines = [
        "DRY-RUN: nenhuma mutacao executada.",
        f"Comando: make {requested_verb} WHAT={what}",
        f"Dominio: {command.domain}",
        f"Resumo: {command.summary}",
        "Regra: comando mutador exige APPLY=Y.",
    ]
    if command.rules:
        lines.extend(["", "Regras aplicadas:"])
        lines.extend(f"  - {rule}" for rule in command.rules)
    if command.params:
        lines.extend(["", "Parametros atuais:"])
        missing: list[Param] = []
        for param in command.params:
            value = param_value(param, command)
            shown = value or "<ausente>"
            required = "obrigatorio" if param.required else "opcional"
            choices = f" choices={','.join(param.choices)}" if param.choices else ""
            lines.append(
                f"  {param.name:24} {shown:24} {required}{choices} - {param.help}"
            )
            if param.required and not value:
                missing.append(param)
        if missing:
            lines.extend(["", "Faltando antes de executar:"])
            lines.extend(f"  {param.name}=<valor>  # {param.help}" for param in missing)
    lines.extend([
        "",
        "Execucao canonica:",
        f"  {example_for(command, requested_verb)}",
        "  # repita com APPLY=Y somente depois de conferir dominio, escopo e bead.",
    ])
    return "\n".join(lines)


def format_params_inline(params: Iterable[Param]) -> str:
    parts: list[str] = []
    for param in params:
        suffix = "*" if param.required else ""
        detail: list[str] = []
        if param.default:
            detail.append(f"default={param.default}")
        if param.choices:
            detail.append(f"choices={','.join(param.choices)}")
        rendered = f"{param.name}{suffix}"
        if detail:
            rendered = f"{rendered}({';'.join(detail)})"
        parts.append(rendered)
    return ", ".join(parts)


def example_for(command: Command, requested_verb: str) -> str:
    canonical = f"make {command.verb}"
    requested = f"make {requested_verb}"
    if (
        requested_verb != command.verb
        and command.example == f"{canonical} WHAT={command.what}"
    ):
        return requested
    if command.example.startswith(canonical):
        return requested + command.example[len(canonical) :]
    return command.example


def env_enabled(name: str) -> bool:
    return os.environ.get(name, "N").upper() in {"1", "Y", "YES", "TRUE"}


def validate_invocation(command: Command, *, require_required: bool = True) -> None:
    for param in command.params:
        value = param_value(param, command)
        if require_required and param.required and not value:
            msg = (
                f"{command.verb} WHAT={command.what}: parametro obrigatorio ausente: "
                f"{param.name}; exemplo: {command.example}"
            )
            raise RegistryError(msg)
        if value and param.choices and value not in param.choices:
            valid = "|".join(param.choices)
            msg = f"{command.verb} WHAT={command.what}: {param.name}={value!r} invalido; validos: {valid}"
            raise RegistryError(msg)


def validate_command_contract(command: Command) -> None:
    param_by_name = {param.name: param for param in command.params}
    if command.mutates:
        ensure_required_params(command, param_by_name, MUTATION_REQUIRED_PARAMS)
        apply_param = param_by_name.get("APPLY")
        if apply_param and "Y" not in apply_param.choices:
            msg = f"{command.path}: APPLY mutador deve declarar choices contendo Y"
            raise RegistryError(msg)
    if command.domain == "incident" and command.mutates:
        ensure_required_params(
            command, param_by_name, INCIDENT_MUTATION_REQUIRED_PARAMS
        )


def validate_all_choices(verb: str, commands: Mapping[str, Command]) -> None:
    all_command = commands["all"]
    what_param = next(
        (param for param in all_command.params if param.name == "WHAT"), None
    )
    if what_param is None or not what_param.choices:
        return
    declared = tuple(sorted(what_param.choices))
    actual = tuple(sorted(commands))
    if declared != actual:
        msg = (
            f"{all_command.path}: choices de WHAT divergem dos comandos promovidos para {verb}: "
            f"declared={','.join(declared)} actual={','.join(actual)}"
        )
        raise RegistryError(msg)


def ensure_required_params(
    command: Command, params: Mapping[str, Param], names: Iterable[str]
) -> None:
    for name in names:
        param = params.get(name)
        if param is None or not param.required:
            msg = f"{command.path}: parametro {name} deve ser obrigatorio"
            raise RegistryError(msg)


def param_value(param: Param, command: Command) -> str:
    if param.name == "WHAT":
        return command.what
    return os.environ.get(param.name, param.default).strip()


def require_dispatched(path: Path) -> None:
    """Fail if a promoted Python command is run outside scripts/dispatch.py."""
    expected = str(path.resolve())
    if (
        os.environ.get("WORKSPACE_COMMAND_DISPATCHED") == "Y"
        and os.environ.get("WORKSPACE_COMMAND_PATH") == expected
    ):
        return
    raise SystemExit(2)


def env_value(name: str, default: str = "") -> str:
    """Return a stripped environment value used by promoted Python commands."""
    return os.environ.get(name, default).strip()


def require_env(name: str, usage: str | None = None) -> str:
    """Return a required environment value or fail with the command contract code."""
    value = env_value(name)
    if value:
        return value
    raise SystemExit(2)


def promoted_main(script_file: str | Path, handler: Callable[[], int]) -> NoReturn:
    """Run a promoted Python command through the canonical dispatcher guard."""
    require_dispatched(Path(script_file))
    raise SystemExit(handler())


if __name__ == "__main__":
    raise SystemExit(main())
