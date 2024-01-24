# typed: false
# frozen_string_literal: true

# DO NOT EDIT, it's a script generated
class Snazy < Formula
  desc "snazy - a snazzy json log viewer"
  homepage "https://github.com/chmouel/snazy"
  version "0.52.17"

  on_macos do
    url "https://github.com/chmouel/snazy/releases/download/0.52.17/snazy-v0.52.17-macos.tar.gz"
    sha256 "a5361ee75c54a0d4ffef56497a67b5a26188ff564cbc50bf4b6a741774d9de94"

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
      url "https://github.com/chmouel/snazy/releases/download/0.52.17/snazy-v0.52.17-linux-amd64.tar.gz"
      sha256 "0b1ae22a07e0576a4e3dbe5123b6cca62de8c96b7aeca7bfbe95ad5ed5d7eab8"

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
      url "https://github.com/chmouel/snazy/releases/download/0.52.17/snazy-v0.52.17-linux-arm64.tar.gz"
      sha256 "cb313e19b11f3383db3b8ff9600d2fc80c32143c116f06c811d57bde4725b64b"

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
