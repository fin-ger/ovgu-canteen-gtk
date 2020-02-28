PREFIX ?= /usr
GLIB_COMPILE_SCHEMAS = $(shell $(PKGCONFIG) --variable=glib_compile_schemas gio-2.0)
CARGO_BUILD_ARGS = --release
TARGET_DIR = release

build:
	@cargo build $(CARGO_BUILD_ARGS)

install: build
	@mkdir -p "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@mkdir -p "$(PREFIX)/share/applications/"
	@mkdir -p "$(PREFIX)/bin"
	@install -m 0644 data/ovgu-canteen32.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 data/ovgu-canteen128.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@desktop-file-install -m 0644 --dir="$(PREFIX)/share/applications/" data/gnome-ovgu-canteen.desktop
	@install -s -m 0755 "target/$(TARGET_DIR)/gnome-ovgu-canteen" "$(PREFIX)/bin/"
	@update-desktop-database "$(PREFIX)/share/applications"
	@gtk-update-icon-cache

run:
	$(MAKE) install PREFIX=~/.local CARGO_BUILD_ARGS= TARGET_DIR=debug
	@gnome-ovgu-canteen

uninstall:
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/ovgu-canteen32.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/ovgu-canteen128.svg"
	@rm "$(PREFIX)/share/applications/gnome-ovgu-canteen.desktop"
	@rm "$(PREFIX)/bin/gnome-ovgu-canteen"

clean:
	@rm -r target
