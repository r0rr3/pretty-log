class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.4-fix1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix1/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "0bb15bb967e55b8b37f2aba7762d75419f535c4c1f8834f32ac822fd5edc2f17"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix1/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "1f4abb677093d43a438b38038c192eccdbeffa06bd6ab88f13919cb308b9aa5b"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix1/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "cf05bf7957bb6b3dfb0a21e1c790d33bee6ab2c048a5fe4cb86ba614e0b4b41b"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4-fix1/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "74de16b2efbb494ea645244fcd51cec5b523546f4e7d5d1ca843ed7c7b62d9f6"
    end
  end

  def install
    bin.install "pretty"
  end

  test do
    output = shell_output("echo '{\"level\":\"info\",\"msg\":\"hello\"}' | #{bin}/pretty --no-color")
    assert_match "INFO", output
    assert_match "hello", output
  end
end
