# typed: false
# frozen_string_literal: true

# DO NOT EDIT, it's a script generated
class Snazy < Formula
  desc "snazy - a snazzy json log viewer"
  homepage "https://github.com/chmouel/snazy"
  version "0.57.3"

  on_macos do
    url "https://github.com/chmouel/snazy/releases/download/0.57.3/snazy-v0.57.3-macos.tar.gz"
    sha256 "8983c47fdbff3848e76359ef0f9fe2f6e11c8e5dbaa87bcd950a36cb3c390082"

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
      url "https://github.com/chmouel/snazy/releases/download/0.57.3/snazy-v0.57.3-linux-amd64.tar.gz"
      sha256 "fceba59732bc6f08eb2ecc4b6c958cd0986dd04fa868df373553814c62b49fd6"

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
      url "https://github.com/chmouel/snazy/releases/download/0.57.3/snazy-v0.57.3-linux-arm64.tar.gz"
      sha256 "0b5bfafbfd348c1034e18e07c757e24e50c57eaab4eb9ac9a434e141327d3881"

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
