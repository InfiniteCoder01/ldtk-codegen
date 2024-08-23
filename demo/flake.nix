{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = (import nixpkgs { inherit system; });
        in
        {
          devShell = with pkgs; mkShell rec {
            xpackages = with pkgs.xorg; [
              libXrandr
              libXinerama
              libXcursor
              libXi
              libX11
            ];
            packages = with pkgs; [
              rustup
              cmake ninja
              clang
              pkg-config
              wayland
              glfw
            ] ++ xpackages;
            LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath ([
              libGL
            ] ++ xpackages);
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          };
          formatter = pkgs.nixpkgs-fmt;
        }
      );
}
