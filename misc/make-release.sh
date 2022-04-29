#!/usr/bin/env bash
set -euf
VERSION=${1-""}
CARGO_VERSION=$(grep '^version = \"' Cargo.toml|grep -Eo '[0-9]+\.[0-9]+\.[0-9]+')
PKGNAME=$(grep '^name = \"' Cargo.toml|sed -E 's/.*"(\w+)"/\1/')

bumpversion() {
   current=$(git describe --tags $(git rev-list --tags --max-count=1))
   echo "Current tag version is ${current}"

   major=$(python3 -c "import semver,sys;print(str(semver.VersionInfo.parse(sys.argv[1]).bump_major()))" ${current})
   minor=$(python3 -c "import semver,sys;print(str(semver.VersionInfo.parse(sys.argv[1]).bump_minor()))" ${current})
   patch=$(python3 -c "import semver,sys;print(str(semver.VersionInfo.parse(sys.argv[1]).bump_patch()))" ${current})

   echo "If we bump we get, Major: ${major} Minor: ${minor} Patch: ${patch}"
   read -p "To which version you would like to bump [M]ajor, Mi[n]or, [P]atch or Manua[l]: " ANSWER
   if [[ ${ANSWER,,} == "m" ]];then
       mode="major"
   elif [[ ${ANSWER,,} == "n" ]];then
       mode="minor"
   elif [[ ${ANSWER,,} == "p" ]];then
       mode="patch"
   elif [[ ${ANSWER,,} == "l" ]];then
       read -p "Enter version: " -e VERSION
       return
   else
       print "no or bad reply??"
       exit
   fi
   VERSION=$(python3 -c "import semver,sys;print(str(semver.VersionInfo.parse(sys.argv[1]).bump_${mode}()))" ${current})
   [[ -z ${VERSION} ]] && {
       echo "could not bump version automatically"
       exit
   }
   echo "[release] Releasing ${VERSION}"
}

[[ $(git rev-parse --abbrev-ref HEAD) != main ]] && {
    echo "you need to be on the main branch"
    exit 1
}
[[ -z ${VERSION} ]] && bumpversion

vfile=Cargo.toml
sed -i "s/^version = .*/version = \"${VERSION}\"/" ${vfile}
cargo build
git commit -S -m "Release ${VERSION} 🥳" ${vfile} Cargo.lock || true
git tag -s ${VERSION} -m "Releasing version ${VERSION}"
git push --tags origin ${VERSION}
git push origin main --no-verify
