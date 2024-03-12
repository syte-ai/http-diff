#!/bin/sh

set -e

version="$1"
mac_sha="$2"
linux_sha="$3"

cat > pkg/brew/http-diff-bin.rb << EOF
class HttpDiffBin < Formula
  version '$version'
  desc "CLI tool to verify consistency across web server versions."
  homepage "https://github.com/syte-ai/http-diff"

  if OS.mac?
      url "https://github.com/syte-ai/http-diff/releases/download/#{version}/http-diff-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "$mac_sha"
  elsif OS.linux?
      url "https://github.com/syte-ai/http-diff/releases/download/#{version}/http-diff-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "$linux_sha"
  end

  conflicts_with "http-diff"

  def install
    bin.install "http-diff"
    man1.install "doc/http-diff.1"

    bash_completion.install "complete/http-diff.bash"
    zsh_completion.install "complete/_http-diff"
  end
end
EOF