#!/usr/bin/env bash
set -euo pipefail

main() {
	local target_dir="assets/admin"
	local archive_name="admin_panel.tar.gz"
	local url="https://github.com/SeaQL/sea-orm-pro/releases/latest/download/${archive_name}"

	rm -rf "${target_dir}"
	mkdir -p "${target_dir}"

	curl -sSfL "${url}" -o "${target_dir}/${archive_name}"
	tar xf "${target_dir}/${archive_name}" --strip-components 1 -C "${target_dir}"
	rm -f "${target_dir}/${archive_name}"

	printf 'Admin frontend downloaded to %s\n' "${target_dir}"
}

main "$@"
