class HttpDiff < Formula
  version '0.0.4'
  desc "CLI tool to verify consistency across web server versions."
  homepage "https://github.com/syte-ai/http-diff"

  if OS.mac?
      url "https://github.com/syte-ai/http-diff/releases/download/#{version}/http-diff-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "59ec99fcc86065ccee2b42c894fee244ff188400dd8840cd5ae5987b5a4f24e4"
  elsif OS.linux?
      url "https://github.com/syte-ai/http-diff/releases/download/#{version}/http-diff-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "13b03fc42ad8533df6b64a057fed261455ae33a02b7f59a8804ccf354de7b303"
  end

  conflicts_with "http-diff"

  def install
    bin.install "http-diff"
    man1.install "doc/http-diff.1"

    bash_completion.install "complete/http-diff.bash"
    zsh_completion.install "complete/_http-diff"
  end
end
