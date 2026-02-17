final: prev:
let
  version = "1.1.0";

  assets = {
    x86_64-linux = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-linux-x86_64.tar.gz";
      hash = "sha256-JvH3NXH07D+6apL0BI9GNYcHtIrUFlj1/O0QfhCl2GE=";
    };
    aarch64-linux = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-linux-aarch64.tar.gz";
      hash = "sha256-S4D5ax+lJeCRpgjitMMZ4ZtYf8ybAhO5SHkMfvBQeC4=";
    };
    x86_64-darwin = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-darwin-x86_64.tar.gz";
      hash = "sha256-gt33Sh3Q5xwxC9OHFcmoA3FkN7/kk6yss7uCOzN7Tu0=";
    };
    aarch64-darwin = {
      url = "https://github.com/taskbook-sh/taskbook/releases/download/v${version}/tb-darwin-aarch64.tar.gz";
      hash = "sha256-J1NTmo0zWkECxXr+7htCXyA9CRZd97UHzzjPrk3JhqE=";
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
