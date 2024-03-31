class HttpDiff < Formula
  version '0.0.3'
  desc "CLI tool to verify consistency across web server versions."
  homepage "https://github.com/syte-ai/http-diff"

  if OS.mac?
      url "https://github.com/syte-ai/http-diff/releases/download/#{version}/http-diff-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "b0001ad91e9b1d759a2f23627deda5d0da4cf638e81046c9b999ecc009626059"
  elsif OS.linux?
      url "https://github.com/syte-ai/http-diff/releases/download/#{version}/http-diff-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "d03e506ea0cb15e4d96feabb55ae3c7521ade9b3a8c2a3dea7341d28765b7625"
  end

  conflicts_with "http-diff"

  def install
    bin.install "http-diff"
    man1.install "doc/http-diff.1"

    bash_completion.install "complete/http-diff.bash"
    zsh_completion.install "complete/_http-diff"
  end
end
