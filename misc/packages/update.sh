#!/usr/bin/env bash
# Copyright 2024 Chmouel Boudjnah <chmouel@chmouel.com>
set -euxfo pipefail
current_dir=$(cd "$(dirname "$0")" && pwd)

project_name=snazy
project_url=https://github.com/chmouel/${project_name}
cargo_release_version=

get_sha256() {
	curl -sSL "$1" | shasum -a 256 | awk '{print $1}'
}

[[ -e Cargo.toml ]] && {
	cargo_release_version=$(grep '^version = "' Cargo.toml | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+')
}

VERSION=${1:-$cargo_release_version}
MACOS_URL="${project_url}/releases/download/${VERSION}/${project_name}-v${VERSION}-macos.tar.gz"
LINUX_URL="${project_url}/releases/download/${VERSION}/${project_name}-v${VERSION}-linux-amd64.tar.gz"
LINUX_ARM_URL="${project_url}/releases/download/${VERSION}/${project_name}-v${VERSION}-linux-arm64.tar.gz"

MACOS_SHA256=$(get_sha256 "$MACOS_URL")
LINUX_SHA256=$(get_sha256 "$LINUX_URL")
LINUX_ARM_SHA256=$(get_sha256 "$LINUX_ARM_URL")

function update_brew() {
	sed -e "s,%VERSION%,${VERSION},g" \
		-e "s,%MACOS_URL%,${MACOS_URL},g" \
		-e "s,%MACOS_SHA256%,${MACOS_SHA256},g" \
		-e "s,%LINUX_URL%,${LINUX_URL},g" \
		-e "s,%LINUX_SHA256%,${LINUX_SHA256},g" \
		-e "s,%LINUX_ARM_URL%,${LINUX_ARM_URL},g" \
		-e "s,%LINUX_ARM_SHA256%,${LINUX_ARM_SHA256},g" <${current_dir}/brews/${project_name}.tmpl.rb \
		>Formula/${project_name}.rb
	[[ -n $(git status -s Formula/${project_name}.rb) ]] && {
		git add Formula/${project_name}.rb
		git commit -m "Formula Bump ${project_name} to ${VERSION}"
		[[ -n ${GITHUB_TOKEN:-""} ]] && git push https://git:${GITHUB_TOKEN}@github.com/chmouel/${project_name} main
	}
}

function update_aur() {
	local ssh_key=/tmp/.arch.aur.key
	[[ -z ${AUR_PRIVATE_KEY:-""} ]] && return
	cat <<EOF >${ssh_key}
${AUR_PRIVATE_KEY}
EOF
	chmod 600 ${ssh_key}
	rm -rf /tmp/${project_name}-bin
	ssh-agent bash -c "ssh-add ${ssh_key};git clone ssh://aur@aur.archlinux.org/${project_name}-bin.git /tmp/${project_name}-bin"
	sed -e "s,%VERSION%,${VERSION},g" \
		-e "s,%LINUX_URL%,${LINUX_URL},g" \
		-e "s,%LINUX_SHA256%,${LINUX_SHA256},g" \
		-e "s,%LINUX_ARM_URL%,${LINUX_ARM_URL},g" \
		-e "s,%LINUX_ARM_SHA256%,${LINUX_ARM_SHA256},g" <${current_dir}/aur/PKGBUILD.tmpl \
		>/tmp/${project_name}-bin/PKGBUILD
	sed -e "s,%VERSION%,${VERSION},g" \
		-e "s,%LINUX_URL%,${LINUX_URL},g" \
		-e "s,%LINUX_SHA256%,${LINUX_SHA256},g" \
		-e "s,%LINUX_ARM_URL%,${LINUX_ARM_URL},g" \
		-e "s,%LINUX_ARM_SHA256%,${LINUX_ARM_SHA256},g" <${current_dir}/aur/SRCINFO.tmpl \
		>/tmp/${project_name}-bin/.SRCINFO
	(
		cd /tmp/${project_name}-bin/ &&
			git add PKGBUILD .SRCINFO &&
			git commit -m "Bump to ${VERSION}" &&
			ssh-agent bash -c "ssh-add ${ssh_key}; git push"
	)
}

update_brew
update_aur
