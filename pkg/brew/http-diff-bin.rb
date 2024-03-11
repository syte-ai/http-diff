class HttpDiffBin < Formula
  version '0.0.1'
  desc "CLI tool to verify consistency across web server versions."
  homepage "https://github.com/syte-ai/http-diff"

  if OS.mac?
      url "https://github.com/syte-ai/http-diff/releases/download/#{version}/http-diff-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  elsif OS.linux?
      url "https://github.com/syte-ai/http-diff/releases/download/#{version}/http-diff-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  end

  conflicts_with "http-diff"

  def install
    bin.install "http-diff"
    man1.install "doc/http-diff.1"

    bash_completion.install "complete/http-diff.bash"
    zsh_completion.install "complete/_http-diff"
  end
end
