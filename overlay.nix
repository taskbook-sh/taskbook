final: prev:
let
  version = "1.2.0";

  assets = {
    x86_64-linux = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-linux-x86_64.tar.gz";
      hash = "sha256-3rJho8udsWEOI9MhLH5F5pj8rBrhQKlz22JTEkciuCA=";
    };
    aarch64-linux = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-linux-aarch64.tar.gz";
      hash = "sha256-7ZYk/vacyXOJj2XyoyVkgG0+9XSIbjXRkDJkBpPMnTQ=";
    };
    x86_64-darwin = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-darwin-x86_64.tar.gz";
      hash = "sha256-5fPLMVWv5reVrepqH0nFXf43BJAy79xfyiOFsW4m37o=";
    };
    aarch64-darwin = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-darwin-aarch64.tar.gz";
      hash = "sha256-QCS/sZyNpTK1immtUymOkGunwKkDGuixDRTQdExrwjg=";
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
