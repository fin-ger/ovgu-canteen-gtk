PREFIX ?= /usr
GLIB_COMPILE_SCHEMAS = $(shell $(PKGCONFIG) --variable=glib_compile_schemas gio-2.0)

build-dev:
	@cargo build

build:
	@cargo build --release

install-dev: build-dev
	@mkdir -p "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@mkdir -p "$(PREFIX)/share/applications/"
	@mkdir -p "$(PREFIX)/bin"
	@install -m 0644 data/ovgu-canteen32.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 data/ovgu-canteen128.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@desktop-file-install -m 0644 --dir="$(PREFIX)/share/applications/" data/gnome-ovgu-canteen.desktop
	@install -m 0755 target/debug/gnome-ovgu-canteen "$(PREFIX)/bin/"
	@update-desktop-database "$(PREFIX)/share/applications"

install: build
	@mkdir -p "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@mkdir -p "$(PREFIX)/share/applications/"
	@mkdir -p "$(PREFIX)/bin"
	@install -m 0644 data/ovgu-canteen32.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 data/ovgu-canteen128.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@desktop-file-install -m 0644 --dir="$(PREFIX)/share/applications/" data/gnome-ovgu-canteen.desktop
	@install -s -m 0755 target/release/gnome-ovgu-canteen "$(PREFIX)/bin/"
	@update-desktop-database "$(PREFIX)/share/applications"

run: install-dev
	@gnome-ovgu-canteen

uninstall:
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/ovgu-canteen32.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/ovgu-canteen128.svg"
	@rm "$(PREFIX)/share/applications/gnome-ovgu-canteen.desktop"
	@rm "$(PREFIX)/bin/gnome-ovgu-canteen"

clean:
	@rm -r target
