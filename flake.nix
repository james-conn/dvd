{
	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs/release-25.05";
	};

	outputs = { self, nixpkgs }:
		let pkgs = nixpkgs.legacyPackages.x86_64-linux;
		libpath = pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ];
		in {
			devShells.x86_64-linux.default = pkgs.mkShell {
				packages = with pkgs; [
					cargo
					clippy
					pkg-config
					glib
					gst_all_1.gstreamer
					gst_all_1.gst-plugins-base
					gst_all_1.gst-plugins-good
				];

				env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${libpath}";
			};
		};
}
