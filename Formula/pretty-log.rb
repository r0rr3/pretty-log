class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.4"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "13ac1fed2bc1b30dd3412e850f5505316797343e8853ab012c4b9f69fa82e656"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "cb69bc9a3d04a6b7fad52e15a7ff6181c53ae1e024db2a0dbd0a3c82c16bb360"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "9eea392eb3248edc1eae453879d79056fd1d65c4160275c5f05a7ee8fd2384f6"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.4/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "dde36bbee5bf5c29de49c2d27588fcffa5648e291cee0b9eb47145fcca1731b3"
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
