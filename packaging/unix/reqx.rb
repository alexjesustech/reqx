class Reqx < Formula
  desc "CLI-first API client for developers"
  homepage "https://reqx.dev"
  license "MPL-2.0"
  version "0.1.0"

  on_macos do
    on_intel do
      url "https://github.com/reqx/reqx/releases/download/v0.1.0/reqx-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
    on_arm do
      url "https://github.com/reqx/reqx/releases/download/v0.1.0/reqx-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/reqx/reqx/releases/download/v0.1.0/reqx-x86_64-unknown-linux-musl.tar.gz"
      sha256 "PLACEHOLDER"
    end
    on_arm do
      url "https://github.com/reqx/reqx/releases/download/v0.1.0/reqx-aarch64-unknown-linux-musl.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "reqx"
  end

  test do
    system "#{bin}/reqx", "--version"
  end
end
