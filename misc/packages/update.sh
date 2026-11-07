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
# Fine-grained PAT or GitHub App token. The default GITHUB_TOKEN cannot push to
# main (protected branch) nor trigger the PR checks required by auto-merge.
AUTOMATION_TOKEN=${AUTOMATION_TOKEN:-""}
VERSION=${1:-$cargo_release_version}
MACOS_URL="${project_url}/releases/download/${VERSION}/${project_name}-v${VERSION}-macos.tar.gz"
MACOS_ARM_URL="${project_url}/releases/download/${VERSION}/${project_name}-v${VERSION}-macos-arm64.tar.gz"
LINUX_URL="${project_url}/releases/download/${VERSION}/${project_name}-v${VERSION}-linux-amd64.tar.gz"
LINUX_ARM_URL="${project_url}/releases/download/${VERSION}/${project_name}-v${VERSION}-linux-arm64.tar.gz"

MACOS_SHA256=$(get_sha256 "$MACOS_URL")
MACOS_ARM_SHA256=$(get_sha256 "$MACOS_ARM_URL")
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

# main is protected, so the bump goes through a throwaway branch and a PR that
# auto-merges once the required checks pass.
function open_formula_pr() {
	local branch=ci/formula-bump-${VERSION}
	local remote=https://git:${AUTOMATION_TOKEN}@github.com/chmouel/${project_name}
	git checkout -B "${branch}"
	git push --force "${remote}" "${branch}"
	GH_TOKEN=${AUTOMATION_TOKEN} GITHUB_TOKEN=${AUTOMATION_TOKEN} \
		gh pr create --repo chmouel/${project_name} --base main --head "${branch}" \
		--title "Formula Bump ${project_name} to ${VERSION}" \
		--body "Automated Homebrew formula bump for \`${VERSION}\`." ||
		echo "Reusing existing pull request for ${branch}"
	GH_TOKEN=${AUTOMATION_TOKEN} GITHUB_TOKEN=${AUTOMATION_TOKEN} \
		gh pr merge --repo chmouel/${project_name} --auto --merge "${branch}"
}

function update_brew() {
	targetFile=Formula/${project_name}.rb
	sed -e "s,%VERSION%,${VERSION},g" \
		-e "s,%MACOS_URL%,${MACOS_URL},g" \
		-e "s,%MACOS_SHA256%,${MACOS_SHA256},g" \
		-e "s,%MACOS_ARM_URL%,${MACOS_ARM_URL},g" \
		-e "s,%MACOS_ARM_SHA256%,${MACOS_ARM_SHA256},g" \
		-e "s,%LINUX_URL%,${LINUX_URL},g" \
		-e "s,%LINUX_SHA256%,${LINUX_SHA256},g" \
		-e "s,%LINUX_ARM_URL%,${LINUX_ARM_URL},g" \
		-e "s,%LINUX_ARM_SHA256%,${LINUX_ARM_SHA256},g" <${current_dir}/brews/${project_name}.tmpl.rb \
		>${targetFile}

	if [[ -n $(git status -s ${targetFile}) ]]; then
		git add ${targetFile}
		git commit -m "Formula Bump ${project_name} to ${VERSION}"
		if [[ -n ${AUTOMATION_TOKEN} ]]; then
			open_formula_pr
		elif [[ -n ${GITHUB_TOKEN:-""} ]]; then
			git push
		fi
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
	if ! grep -q aur.archlinux.org ~/.ssh/known_hosts >/dev/null 2>/dev/null; then
		mkdir -p ~/.ssh/
		chmod 0700 ~/.ssh
		ssh-keyscan aur.archlinux.org >>~/.ssh/known_hosts
	fi
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
