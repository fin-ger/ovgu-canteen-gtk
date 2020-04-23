PREFIX ?= /usr
CARGO_BUILD_ARGS ?= --release
TARGET_DIR = release

build:
	@cargo build $(CARGO_BUILD_ARGS)

install: build
	@mkdir -p "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@mkdir -p "$(PREFIX)/share/applications/"
	@mkdir -p "$(PREFIX)/bin"
	@install -m 0644 data/de.fin_ger.OvGUCanteen.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 data/de.fin_ger.OvGUCanteen.About.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@install -m 0644 data/de.fin_ger.OvGUCanteen.Closed.svg "$(PREFIX)/share/icons/hicolor/scalable/apps/"
	@desktop-file-install -m 0644 --dir="$(PREFIX)/share/applications/" data/de.fin_ger.OvGUCanteen.desktop
	@install -s -m 0755 "target/$(TARGET_DIR)/gnome-ovgu-canteen" "$(PREFIX)/bin/"
	@update-desktop-database "$(PREFIX)/share/applications"
	@gtk-update-icon-cache

run:
	@$(MAKE) -s install PREFIX=$(HOME)/.local CARGO_BUILD_ARGS= TARGET_DIR=debug
	@gnome-ovgu-canteen

uninstall:
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.About.svg"
	@rm "$(PREFIX)/share/icons/hicolor/scalable/apps/de.fin_ger.OvGUCanteen.Closed.svg"
	@rm "$(PREFIX)/share/applications/de.fin_ger.OvGUCanteen.desktop"
	@rm "$(PREFIX)/bin/gnome-ovgu-canteen"

clean:
	@$(MAKE) -s uninstall PREFIX=$(HOME)/.local
	@echo "Installation files have been cleaned. To also clean cargo build files run 'cargo clean'"
