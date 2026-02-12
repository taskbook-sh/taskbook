final: prev:
let
  version = "1.0.8";

  assets = {
    x86_64-linux = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-linux-x86_64.tar.gz";
      hash = "sha256-cjB9hok6wcIhrxSwfJ+RWN342w0Ui7VHArVNaXo4oV0=";
    };
    aarch64-linux = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-linux-aarch64.tar.gz";
      hash = "sha256-Qmnvhg21D9J67LORuk+x3CMxD4bghP4px5vDW34aakc=";
    };
    x86_64-darwin = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-darwin-x86_64.tar.gz";
      hash = "sha256-QQrfzzC+N2e22eyKSQSuAsutJExd0U9rHneRv00TU0Q=";
    };
    aarch64-darwin = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-darwin-aarch64.tar.gz";
      hash = "sha256-BM7Q5f7pDZbKWYKfCtEGhWndPjDtOkd9GrMKXjtOujc=";
    };
  };

  asset = assets.${final.stdenv.hostPlatform.system} or (throw "unsupported system: ${final.stdenv.hostPlatform.system}");
in
{
  taskbook = final.stdenv.mkDerivation {
    pname = "taskbook";
    inherit version;

    src = final.fetchurl {
      inherit (asset) url hash;
    };

    sourceRoot = ".";

    unpackPhase = ''
      tar xzf $src
    '';

    installPhase = ''
      install -Dm755 tb $out/bin/tb
    '';

    meta = with final.lib; {
      description = "Tasks, boards & notes for the command-line habitat";
      homepage = "https://github.com/taskbook-sh/taskbook";
      license = licenses.mit;
      mainProgram = "tb";
      platforms = builtins.attrNames assets;
    };
  };
}
