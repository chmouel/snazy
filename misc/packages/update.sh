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

COMMIT_AUTHOR=${COMMIT_AUTHOR:-"Chmouel Boudjnah"}
COMMIT_EMAIL=${COMMIT_EMAIL:-"chmouel@chmouel.com"}
VERSION=${1:-$cargo_release_version}
MACOS_URL="${project_url}/releases/download/${VERSION}/${project_name}-v${VERSION}-macos.tar.gz"
LINUX_URL="${project_url}/releases/download/${VERSION}/${project_name}-v${VERSION}-linux-amd64.tar.gz"
LINUX_ARM_URL="${project_url}/releases/download/${VERSION}/${project_name}-v${VERSION}-linux-arm64.tar.gz"

MACOS_SHA256=$(get_sha256 "$MACOS_URL")
LINUX_SHA256=$(get_sha256 "$LINUX_URL")
LINUX_ARM_SHA256=$(get_sha256 "$LINUX_ARM_URL")

function set_git_config() {
	# make sure git config user.email and user.name is set if not set it
	if [[ -z $(git config --global --get user.email) ]]; then
		git config --global user.email $COMMIT_EMAIL
	fi
	if [[ -z $(git config --global --get user.name) ]]; then
		git config --global user.name $COMMIT_AUTHOR
	fi
}

function update_brew() {
	targetFile=Formula/${project_name}.rb
	if [[ -n ${GITHUB_TOKEN:-""} ]]; then
		rm -rf /tmp/pac-repo
		git clone --depth=1 https://git:${GITHUB_TOKEN}@github.com/chmouel/${project_name} /tmp/pac-repo
		targetFile=/tmp/pac-repo/${targetFile}
		cd /tmp/pac-repo
	fi
	sed -e "s,%VERSION%,${VERSION},g" \
		-e "s,%MACOS_URL%,${MACOS_URL},g" \
		-e "s,%MACOS_SHA256%,${MACOS_SHA256},g" \
		-e "s,%LINUX_URL%,${LINUX_URL},g" \
		-e "s,%LINUX_SHA256%,${LINUX_SHA256},g" \
		-e "s,%LINUX_ARM_URL%,${LINUX_ARM_URL},g" \
		-e "s,%LINUX_ARM_SHA256%,${LINUX_ARM_SHA256},g" <${current_dir}/brews/${project_name}.tmpl.rb \
		>${targetFile}

	if [[ -n $(git status -s Formula/${project_name}.rb) ]]; then
		git add Formula/${project_name}.rb
		git commit -m "Formula Bump ${project_name} to ${VERSION}"
		[[ -n ${GITHUB_TOKEN:-""} ]] && git push
	fi
}

function update_aur() {
	local ssh_key=/tmp/.arch.aur.key
	[[ -z ${AUR_PRIVATE_KEY:-""} ]] && return
	cat <<EOF >${ssh_key}
${AUR_PRIVATE_KEY}
EOF
	chmod 600 ${ssh_key}
	rm -rf /tmp/${project_name}-bin
	grep -q aur.archlinux.org ~/.ssh/known_hosts >/dev/null 2>/dev/null || ssh-keyscan aur.archlinux.org >>~/.ssh/known_hosts
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

set_git_config
update_brew
update_aur
