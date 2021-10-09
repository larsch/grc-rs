all:
	cargo build

release-build: grc-rs.1.gz
	cargo build --release

test:
	cargo test

install:
	install -D -m 0755 target/release/grc-rs $(DESTDIR)/usr/bin/grc-rs
	install -D -m 0644 grc-rs.1.gz $(DESTDIR)/usr/share/man/man1/grc-rs.1.gz
	install -D -m 0644 zsh.compl $(DESTDIR)/usr/share/zsh/functions/Completion/Zsh/_grc-rs
	install -D -m 0644 config/grc-rs.conf $(DESTDIR)/etc/grc-rs.conf
	install -D -m 0644 config/conf.lsusb $(DESTDIR)/usr/share/grc-rs/conf.lsusb

 grc-rs.1.gz:  grc-rs.1
	gzip -k grc-rs.1
