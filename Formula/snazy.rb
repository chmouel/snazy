# typed: false
# frozen_string_literal: true

# Generated with script : misc/formula-bump/update-formula.sh
# DO NOT EDIT
class Snazy < Formula
  desc "snazy - a snazzy json log viewer"
  homepage "https://github.com/chmouel/snazy"
  version "0.52.11"

  on_macos do
    url "https://github.com/chmouel/snazy/releases/download/0.52.11/snazy-v0.52.11-macos.tar.gz"
    sha256 "a6539b364c63717c82fb55a8976006b4cfcdbd15b45b92434c188f178bd44440"

    def install
      bin.install "snazy" => "snazy"
      prefix.install_metafiles

      output = Utils.popen_read("SHELL=bash #{bin}/snazy --shell-completion bash")
      (bash_completion/"snazy").write output

      output = Utils.popen_read("SHELL=zsh #{bin}/snazy --shell-completion zsh")
      (zsh_completion/"_snazy").write output

      output = Utils.popen_read("SHELL=fish #{bin}/snazy --shell-completion fish")
      (fish_completion/"snazy.fish").write output
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/chmouel/snazy/releases/download/0.52.11/snazy-v0.52.11-linux-amd64.tar.gz"
      sha256 "9d04c46536267229abf75f5d1193a8de960d36747e6ea8061b259f8f3a787f46"

      def install
        bin.install "snazy" => "snazy"
        prefix.install_metafiles

        output = Utils.popen_read("SHELL=bash #{bin}/snazy --shell-completion bash")
        (bash_completion/"snazy").write output

        output = Utils.popen_read("SHELL=zsh #{bin}/snazy --shell-completion zsh")
        (zsh_completion/"_snazy").write output

        output = Utils.popen_read("SHELL=fish #{bin}/snazy --shell-completion fish")
        (fish_completion/"snazy.fish").write output
      end
    end
    if Hardware::CPU.arm? && Hardware::CPU.is_64_bit?
      url "https://github.com/chmouel/snazy/releases/download/0.52.11/snazy-v0.52.11-linux-arm64.tar.gz"
      sha256 "89ba17979ae04e366b8ae90aa8bfc7a9fc499e9d376aa30c54f6a109f4992e64"

      def install
        bin.install "snazy" => "snazy"
        prefix.install_metafiles

        output = Utils.popen_read("SHELL=bash #{bin}/snazy --shell-completion bash")
        (bash_completion/"snazy").write output

        output = Utils.popen_read("SHELL=zsh #{bin}/snazy --shell-completion zsh")
        (zsh_completion/"_snazy").write output

        output = Utils.popen_read("SHELL=fish #{bin}/snazy --shell-completion fish")
        (fish_completion/"snazy.fish").write output
      end
    end
  end
end
