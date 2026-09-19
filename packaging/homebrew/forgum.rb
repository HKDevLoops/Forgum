class Forgum < Formula
  desc "Cross-platform ANSI animation mascot and shell integration engine"
  homepage "https://github.com/HKDevLoops/Forgum"
  license "MIT"

  version "0.4.0"

  stable do
    on_macos do
      on_intel do
        url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-x86_64-apple-darwin.tar.gz"
        sha256 "PLACEHOLDER_SHA256"
      end
      on_arm do
        url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-aarch64-apple-darwin.tar.gz"
        sha256 "PLACEHOLDER_SHA256"
      end
    end
    on_linux do
      on_intel do
        url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-x86_64-unknown-linux-gnu.tar.gz"
        sha256 "PLACEHOLDER_SHA256"
      end
      on_arm do
        url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-aarch64-unknown-linux-gnu.tar.gz"
        sha256 "PLACEHOLDER_SHA256"
      end
    end
  end

  def install
    bin.install "forgum"
    bin.install "forgum-engine" if File.exist?("forgum-engine")

    # Generate and install shell completions
    generate_completions_from_executable(bin/"forgum", "completions", shells: [:bash, :zsh, :fish])
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/forgum --version")
  end
end
