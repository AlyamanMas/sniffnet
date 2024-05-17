{
  inputs = { utils.url = "github:numtide/flake-utils"; };
  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ pkg-config ];

          buildInputs = with pkgs;
            [ libpcap openssl ] ++ lib.optionals stdenv.isLinux [
              alsa-lib
              libcgroup
              expat
              fontconfig
              vulkan-loader
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
            ] ++ lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.AppKit
              rustPlatform.bindgenHook
            ];
        };
      });
}
