class HttpDiff < Formula
  version '0.0.5'
  desc "CLI tool to verify consistency across web server versions."
  homepage "https://github.com/syte-ai/http-diff"

  if OS.mac?
      url "https://github.com/syte-ai/http-diff/releases/download/#{version}/http-diff-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "9cffe02455e1cd45056d32c225c2497a25bf1b2c050fa101e3778865bd09388c"
  elsif OS.linux?
      url "https://github.com/syte-ai/http-diff/releases/download/#{version}/http-diff-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "92ea65bb3eb2b2ff5649975009643f0d655c11f52ef54ce7f226995c9de52eca"
  end

  conflicts_with "http-diff"

  def install
    bin.install "http-diff"
    man1.install "doc/http-diff.1"

    bash_completion.install "complete/http-diff.bash"
    zsh_completion.install "complete/_http-diff"
  end
end
