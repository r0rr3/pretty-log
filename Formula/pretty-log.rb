class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.1.0/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "5a12dcca53fae1b3fb8fb60d2f50cf7a83b3daffc56fc52692f5b83d3075d370"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.1.0/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "b150363a129d475f8a7971e1ca0af1ed3d93f76cea5788e67a42240e7d854124"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.1.0/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "1169404227bb093fce934a78cfedf8ac9ad6b8a5f631838767ad2ae11ffd5288"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.1.0/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "ae5ee03b483bc641731ec37b3cfd93c93b201739011274f692a3c27fca6c264b"
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
