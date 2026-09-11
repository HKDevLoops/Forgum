class Forgum < Formula
  desc "Cross-platform cowsay+fortune+lolcat with a Rust ANSI animation engine"
  homepage "https://github.com/HKDevLoops/Forgum"
  license "MIT"

  version "0.0.1-alpha.1"

  stable do
    on_macos do
      on_intel do
        url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-engine-x86_64-apple-darwin.tar.gz"
        sha256 "PLACEHOLDER_SHA256"
      end
      on_arm do
        url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-engine-aarch64-apple-darwin.tar.gz"
        sha256 "PLACEHOLDER_SHA256"
      end
    end
    on_linux do
      on_intel do
        url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-engine-x86_64-unknown-linux-gnu.tar.gz"
        sha256 "PLACEHOLDER_SHA256"
      end
      on_arm do
        url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-engine-aarch64-unknown-linux-gnu.tar.gz"
        sha256 "PLACEHOLDER_SHA256"
      end
    end
  end

  def install
    bin.install "forgum-engine"
    bin.install "forgum" if File.exist?("forgum")

    # Generate and install shell completions
    generate_completions_from_executable(bin/"forgum-engine", "completions", shells: [:bash, :zsh, :fish])
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/forgum-engine --version")
  end
end
