# typed: false
# frozen_string_literal: true

# DO NOT EDIT, it's a script generated
class Snazy < Formula
  desc "snazy - a snazzy json log viewer"
  homepage "https://github.com/chmouel/snazy"
  version "0.56.0"

  on_macos do
    url "https://github.com/chmouel/snazy/releases/download/0.56.0/snazy-v0.56.0-macos.tar.gz"
    sha256 "397610743fe921f62e7c0890a3a8a1a3d55c12e68b5237c8564ae2ba503aaed1"

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
      url "https://github.com/chmouel/snazy/releases/download/0.56.0/snazy-v0.56.0-linux-amd64.tar.gz"
      sha256 "77e7b90ffb97f7b9bac63984e48df5e22d55096522de05d3c401d62fbc34b8e5"

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
      url "https://github.com/chmouel/snazy/releases/download/0.56.0/snazy-v0.56.0-linux-arm64.tar.gz"
      sha256 "18ece8de042f933ffe70ecea85907dd8e8caa578bb2782da1ad585175f1a1e2b"

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
