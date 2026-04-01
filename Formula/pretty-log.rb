class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.2"
  license "MIT"

  on_macos do
    on_arm64 do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "WILL_BE_UPDATED_BY_GITHUB_ACTIONS"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "WILL_BE_UPDATED_BY_GITHUB_ACTIONS"
    end
  end

  on_linux do
    on_arm64 do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "WILL_BE_UPDATED_BY_GITHUB_ACTIONS"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.2/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "WILL_BE_UPDATED_BY_GITHUB_ACTIONS"
    end
  end

  def install
    bin.install "pretty"
  end

  test do
    output = shell_output("echo '{\"level\":\"info\",\"msg\":\"hello\"}' | #{bin}/pretty --no-color")
    assert_match /INFO/, output
    assert_match /hello/, output
  end
end
