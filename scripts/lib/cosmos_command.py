"""Lib Cosmos Command.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""
from __future__ import annotations

import os
import runpy
import subprocess
import sys
import tomllib
from collections.abc import Callable, Iterable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import NoReturn, cast

from .logger import get_logger

logger = get_logger(__name__)

TomlValue = str | int | float | bool | list["TomlValue"] | dict[str, "TomlValue"]
TomlTable = dict[str, TomlValue]

HEADER_START = "/// cosmos-command"
HEADER_END = "///"
COMMAND_SUFFIXES = frozenset({".sh", ".py"})
IGNORED_DIRS = frozenset({"__pycache__", "hooks", "legado", "lib"})
MUTATION_REQUIRED_PARAMS = frozenset({"APPLY"})
INCIDENT_MUTATION_REQUIRED_PARAMS = frozenset({"APPLY", "EMERGENCY", "BREAKING_GLASS_BEAD"})

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
LOCAL_PYTHON = ROOT / ".venv" / "bin" / "python"


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


class RegistryError(Exception):
    """Raised when command metadata or invocation is invalid."""


class Registry:
    """In-memory view discovered from script headers; never a static catalog."""

    def __init__(self) -> None:
        self._commands: dict[str, dict[str, Command]] = {}
        self._aliases: dict[str, str] = {}

    def add(self, command: Command) -> None:
        by_what = self._commands.setdefault(command.verb, {})
        if command.what in by_what:
            raise RegistryError(f"comando duplicado: {command.verb} WHAT={command.what}")
        by_what[command.what] = command
        for alias in command.aliases:
            previous = self._aliases.get(alias)
            if previous and previous != command.verb:
                raise RegistryError(f"alias duplicado: {alias} aponta para {previous} e {command.verb}")
            self._aliases[alias] = command.verb

    def validate(self) -> None:
        if not self._commands:
            raise RegistryError("nenhum comando promovido encontrado em scripts/<verbo>/<WHAT>")
        for verb, commands in sorted(self._commands.items()):
            if "all" not in commands:
                raise RegistryError(f"verbo '{verb}' sem WHAT=all")
            domains = {command.domain for command in commands.values()}
            if len(domains) != 1:
                valid = ", ".join(sorted(domains))
                raise RegistryError(f"verbo '{verb}' declara mais de um domain: {valid}")
            for command in commands.values():
                if command.what != "all" and command.aliases:
                    raise RegistryError(f"{command.path}: aliases devem ser declarados apenas em WHAT=all")
                validate_command_contract(command)
            validate_all_choices(verb, commands)
        for alias in self._aliases:
            if alias in self._commands:
                raise RegistryError(f"alias '{alias}' colide com verbo promovido")

    def verbs(self) -> list[str]:
        return sorted(self._commands)

    def resolve_verb(self, verb: str) -> str:
        resolved = self._aliases.get(verb, verb)
        if resolved not in self._commands:
            raise RegistryError(f"verbo '{verb}' desconhecido")
        return resolved

    def commands(self, verb: str) -> Mapping[str, Command]:
        return self._commands[self.resolve_verb(verb)]

    def command(self, verb: str, what: str) -> Command:
        commands = self.commands(verb)
        if what not in commands:
            valid = " ".join(sorted(commands))
            raise RegistryError(f"WHAT='{what}' invalido para {verb}. Validos: {valid}")
        return commands[what]

    def aliases_for(self, verb: str) -> list[str]:
        return sorted(alias for alias, target in self._aliases.items() if target == verb)


def main(argv: Sequence[str] | None = None) -> int:
    args = tuple(sys.argv[1:] if argv is None else argv)
    try:
        registry = discover()
        if args and args[0] == "--validate":
            return 0
        if not args or args[0] in {"help", "--help", "-h"}:
            requested = os.environ.get("WHAT", "").strip()
            logger.info(render_requested_help(registry, requested))
            return 0
        return dispatch(registry, args[0])
    except RegistryError as exc:
        logger.error(f"ERRO: {exc}")
        return 2


def dispatch(registry: Registry, requested_verb: str) -> int:
    verb = registry.resolve_verb(requested_verb)
    what = os.environ.get("WHAT", "").strip() or "all"
    if what in {"all", "help"}:
        logger.info(render_verb_help(registry, requested_verb))
        return 0
    command = registry.command(verb, what)
    if env_enabled("HELP") or env_enabled("OPTIONS"):
        logger.info(render_command_help(registry, requested_verb, what))
        return 0
    is_dry_run = command.mutates and os.environ.get("APPLY", "N") != "Y"
    validate_invocation(command, require_required=not is_dry_run)
    if is_dry_run:
        logger.info(render_dry_run(command, requested_verb, what))
        return 0
    return run(command)


def run(command: Command) -> int:
    env = command_env(command)
    if command.path.suffix == ".py":
        return run_python(command, env)
    return subprocess.run(["bash", str(command.path)], cwd=ROOT, env=env, check=False).returncode


def run_python(command: Command, env: Mapping[str, str]) -> int:
    previous_env = os.environ.copy()
    previous_argv = sys.argv[:]
    previous_cwd = Path.cwd()
    try:
        os.environ.clear()
        os.environ.update(env)
        sys.argv = [str(command.path)]
        os.chdir(ROOT)
        try:
            runpy.run_path(str(command.path), run_name="__main__")
        except SystemExit as exc:
            code = exc.code
            if code is None:
                return 0
            if isinstance(code, int):
                return code
            logger.error(code)
            return 1
        return 0
    finally:
        os.chdir(previous_cwd)
        sys.argv = previous_argv
        os.environ.clear()
        os.environ.update(previous_env)


def command_env(command: Command) -> dict[str, str]:
    """Return the canonical environment for a promoted command."""

    env = os.environ.copy()
    env["WHAT"] = command.what
    env["COSMOS_COMMAND_DISPATCHED"] = "Y"
    env["COSMOS_COMMAND_VERB"] = command.verb
    env["COSMOS_COMMAND_WHAT"] = command.what
    env["COSMOS_COMMAND_DOMAIN"] = command.domain
    env["COSMOS_COMMAND_PATH"] = str(command.path.resolve())
    env.pop("PYTHONPATH", None)
    return env


def local_python() -> str:
    """Return the repository-local Python command, falling back to the active one."""

    explicit = os.environ.get("COSMOS_PYTHON", "").strip()
    if explicit:
        return explicit
    if LOCAL_PYTHON.is_file():
        return str(LOCAL_PYTHON)
    return sys.executable


def discover() -> Registry:
    registry = Registry()
    if not SCRIPTS.exists():
        raise RegistryError("diretorio scripts ausente")
    verb_dirs = sorted(path for path in SCRIPTS.iterdir() if path.is_dir() and path.name not in IGNORED_DIRS)
    for verb_dir in verb_dirs:
        for path in sorted(verb_dir.iterdir()):
            if path.name == "__pycache__":
                continue
            if path.is_dir():
                continue
            if path.suffix not in COMMAND_SUFFIXES:
                continue
            command = load_command(path, verb_dir.name)
            if command is None:
                continue
            registry.add(command)
    registry.validate()
    return registry


def load_command(path: Path, expected_verb: str) -> Command | None:
    data = header_data(path)
    if data is None:
        return None
    verb = require_string(data, "verb", path)
    what = require_string(data, "what", path)
    if verb != expected_verb:
        raise RegistryError(f"{path}: header verb={verb} diverge do diretorio {expected_verb}")
    if what != path.stem:
        raise RegistryError(f"{path}: header what={what} diverge do arquivo {path.stem}")
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


def header_data(path: Path) -> TomlTable | None:
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
        return None
    try:
        return cast(TomlTable, tomllib.loads("\n".join(payload)))
    except tomllib.TOMLDecodeError as exc:
        raise RegistryError(f"{path}: header TOML invalido: {exc}") from exc


def require_string(data: Mapping[str, TomlValue], key: str, path: Path) -> str:
    value = data.get(key)
    if not isinstance(value, str) or not value.strip():
        raise RegistryError(f"{path}: campo obrigatorio ausente: {key}")
    return value.strip()


def require_bool(data: Mapping[str, TomlValue], key: str, path: Path) -> bool:
    value = data.get(key)
    if not isinstance(value, bool):
        raise RegistryError(f"{path}: campo booleano obrigatorio ausente: {key}")
    return value


def parse_aliases(value: TomlValue | None, path: Path) -> tuple[str, ...]:
    return parse_string_list(value, "aliases", path)


def parse_string_list(value: TomlValue | None, field: str, path: Path) -> tuple[str, ...]:
    if value is None:
        return ()
    if not isinstance(value, list):
        raise RegistryError(f"{path}: {field} deve ser lista de strings")
    values: list[str] = []
    for item in value:
        if not isinstance(item, str) or not item.strip():
            raise RegistryError(f"{path}: {field} invalido")
        values.append(item.strip())
    return tuple(values)


def parse_params(value: TomlValue | None, path: Path) -> tuple[Param, ...]:
    if value is None:
        return ()
    if not isinstance(value, list):
        raise RegistryError(f"{path}: params deve ser lista de objetos")
    params: list[Param] = []
    for item in value:
        if not isinstance(item, dict):
            raise RegistryError(f"{path}: params deve conter objetos TOML")
        params.append(parse_param(item, path))
    return tuple(params)


def parse_param(data: Mapping[str, TomlValue], path: Path) -> Param:
    name = require_string(data, "name", path)
    help_text = require_string(data, "help", path)
    required_raw = data.get("required", False)
    default_raw = data.get("default", "")
    if not isinstance(required_raw, bool):
        raise RegistryError(f"{path}: params.required deve ser booleano")
    if not isinstance(default_raw, str):
        raise RegistryError(f"{path}: params.default deve ser string")
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
    return render_verb_help(registry, requested)


def render_global_help(registry: Registry) -> str:
    lines = ["workspace - make <verbo> WHAT=<acao> [PARAM=value ...]", ""]
    for verb in registry.verbs():
        command = registry.command(verb, "all")
        aliases = registry.aliases_for(verb)
        suffix = f" (alias: {', '.join(aliases)})" if aliases else ""
        lines.append(f"  {verb:14} [{command.domain:12}] {command.summary}{suffix}")
    lines.extend(
        [
            "",
            "make <verbo> mostra o help do verbo e todos os WHAT.",
            "make help WHAT=<verbo> mostra o mesmo help.",
            "make help WHAT=<verbo>/<acao> ou make <verbo> WHAT=<acao> OPTIONS=Y mostra uma acao.",
            "Comandos mutadores exigem APPLY=Y.",
            "Novos comandos vivem em scripts/<verbo>/<WHAT>.sh|py com header cosmos-command.",
        ]
    )
    return "\n".join(lines)


def render_verb_help(registry: Registry, requested_verb: str) -> str:
    verb = registry.resolve_verb(requested_verb)
    aliases = registry.aliases_for(verb)
    alias_suffix = f" (alias: {', '.join(aliases)})" if aliases else ""
    lines = [f"make {requested_verb} WHAT=<WHAT>{alias_suffix}", "", "WHAT disponiveis:"]
    commands = registry.commands(verb)
    for what, command in sorted(commands.items()):
        marker = " [mutates]" if command.mutates else ""
        lines.append(f"  {what:20} [{command.domain:12}] {command.summary}{marker}")
    command_params = [(what, command) for what, command in sorted(commands.items()) if command.params]
    if command_params:
        lines.extend(["", "Opcoes por WHAT:"])
        for what, command in command_params:
            lines.append(f"  {what:20} {format_params_inline(command.params)}")
        lines.extend(
            [
                "",
                "Detalhe de uma acao:",
                f"  make help WHAT={requested_verb}/<WHAT>",
                f"  make {requested_verb} WHAT=<WHAT> OPTIONS=Y",
            ]
        )
    rules = sorted({rule for command in commands.values() for rule in command.rules})
    if rules:
        lines.extend(["", "Regras:"])
        for rule in rules:
            lines.append(f"  - {rule}")
    examples = sorted({example_for(command, requested_verb) for command in commands.values()})
    if examples:
        lines.extend(["", "Exemplos:"])
        for example in examples:
            lines.append(f"  {example}")
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
        for rule in command.rules:
            lines.append(f"  - {rule}")
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
        for rule in command.rules:
            lines.append(f"  - {rule}")
    if command.params:
        lines.extend(["", "Parametros atuais:"])
        missing: list[Param] = []
        for param in command.params:
            value = param_value(param, command)
            shown = value if value else "<ausente>"
            required = "obrigatorio" if param.required else "opcional"
            choices = f" choices={','.join(param.choices)}" if param.choices else ""
            lines.append(f"  {param.name:24} {shown:24} {required}{choices} - {param.help}")
            if param.required and not value:
                missing.append(param)
        if missing:
            lines.extend(["", "Faltando antes de executar:"])
            for param in missing:
                lines.append(f"  {param.name}=<valor>  # {param.help}")
    lines.extend(
        [
            "",
            "Execucao canonica:",
            f"  {example_for(command, requested_verb)}",
            "  # repita com APPLY=Y somente depois de conferir dominio, escopo e bead.",
        ]
    )
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
    if command.example.startswith(canonical):
        return requested + command.example[len(canonical) :]
    return command.example


def env_enabled(name: str) -> bool:
    return os.environ.get(name, "N").upper() in {"1", "Y", "YES", "TRUE"}


def validate_invocation(command: Command, *, require_required: bool = True) -> None:
    for param in command.params:
        value = param_value(param, command)
        if require_required and param.required and not value:
            raise RegistryError(
                f"{command.verb} WHAT={command.what}: parametro obrigatorio ausente: "
                f"{param.name}; exemplo: {command.example}"
            )
        if value and param.choices and value not in param.choices:
            valid = "|".join(param.choices)
            raise RegistryError(
                f"{command.verb} WHAT={command.what}: {param.name}={value!r} invalido; validos: {valid}"
            )


def validate_command_contract(command: Command) -> None:
    param_by_name = {param.name: param for param in command.params}
    if command.mutates:
        ensure_required_params(command, param_by_name, MUTATION_REQUIRED_PARAMS)
        apply_param = param_by_name.get("APPLY")
        if apply_param and "Y" not in apply_param.choices:
            raise RegistryError(f"{command.path}: APPLY mutador deve declarar choices contendo Y")
    if command.domain == "incident" and command.mutates:
        ensure_required_params(command, param_by_name, INCIDENT_MUTATION_REQUIRED_PARAMS)


def validate_all_choices(verb: str, commands: Mapping[str, Command]) -> None:
    all_command = commands["all"]
    what_param = next((param for param in all_command.params if param.name == "WHAT"), None)
    if what_param is None or not what_param.choices:
        return
    declared = tuple(sorted(what_param.choices))
    actual = tuple(sorted(commands))
    if declared != actual:
        raise RegistryError(
            f"{all_command.path}: choices de WHAT divergem dos comandos promovidos para {verb}: "
            f"declared={','.join(declared)} actual={','.join(actual)}"
        )


def ensure_required_params(command: Command, params: Mapping[str, Param], names: Iterable[str]) -> None:
    for name in names:
        param = params.get(name)
        if param is None or not param.required:
            raise RegistryError(f"{command.path}: parametro {name} deve ser obrigatorio")


def param_value(param: Param, command: Command) -> str:
    if param.name == "WHAT":
        return command.what
    return os.environ.get(param.name, param.default).strip()


def require_dispatched(path: Path) -> None:
    """Fail if a promoted Python command is run outside scripts/dispatch.py."""

    expected = str(path.resolve())
    if os.environ.get("COSMOS_COMMAND_DISPATCHED") == "Y" and os.environ.get("COSMOS_COMMAND_PATH") == expected:
        return
    logger.error("ERRO: comandos publicos devem ser executados via make <verbo> WHAT=<acao>")
    raise SystemExit(2)


def env_value(name: str, default: str = "") -> str:
    """Return a stripped environment value used by promoted commands."""

    return os.environ.get(name, default).strip()


def require_env(name: str, usage: str | None = None) -> str:
    """Return a required environment value or fail with the command contract code."""

    value = env_value(name)
    if value:
        return value
    label = usage or f"{name}=<valor>"
    logger.error(f"ERRO: {label} obrigatorio")
    raise SystemExit(2)


def promoted_main(script_file: str | Path, handler: Callable[[], int]) -> NoReturn:
    """Run a promoted Python command through the canonical dispatcher guard."""

    require_dispatched(Path(script_file))
    raise SystemExit(handler())


if __name__ == "__main__":
    raise SystemExit(main())
