class PrettyLog < Formula
  desc "A fast log prettifier for JSON logs with streaming support"
  homepage "https://github.com/r0rr3/pretty-log"
  version "0.0.5"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.5/pretty-log-aarch64-apple-darwin.tar.gz"
      sha256 "78101d8d58b5fc323159855687f53b32ae12036258f7ad3f98a4a11ee66f2568"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.5/pretty-log-x86_64-apple-darwin.tar.gz"
      sha256 "de3a131a8f17a2a306acbeae081b8e7d2f28db72bf9a498a85e925ae901f5d80"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.5/pretty-log-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "b41bf253c6926c3148739a4e4092e002e7ef90db2f1d18a1a400257fbcf0c1ad"
    end
    on_intel do
      url "https://github.com/r0rr3/pretty-log/releases/download/v0.0.5/pretty-log-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "596c8bd3a013a13267a9144d2d08e9e10a59f1806f5ad69eeb35febebbaca38a"
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
