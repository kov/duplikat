DESTDIR ?=
prefix ?= /usr/local

all: duplikat duplikatd

duplikat: target/release/duplikat
target/release/duplikat: duplikat/* duplikat/src/* duplikat-types/* duplikat-types/src/*
	env DUPLIKAT_PREFIX=$(prefix) cargo build --release --bin duplikat

duplikatd: target/release/duplikatd
target/release/duplikatd: duplikat/* duplikat/src/* duplikat-types/* duplikat-types/src/* target/release/duplikatd.service
	env DUPLIKAT_PREFIX=$(prefix) cargo build --release --bin duplikatd

target/release/duplikatd.service: duplikatd/duplikatd.service.in
	sed 's,@@PREFIX@@,$(prefix),g' $< > $@

clean:
	cargo clean

test:
	cargo test

clippy:
	cargo clippy

install: all
	install -m 755 -d $(DESTDIR)$(prefix)/bin
	install -m 755 target/release/duplikat $(DESTDIR)$(prefix)/bin/
	install -m 755 -d $(DESTDIR)$(prefix)/sbin
	install -m 755 target/release/duplikatd $(DESTDIR)$(prefix)/sbin/
	install -m 755 -d $(DESTDIR)$(prefix)/lib/systemd/system/
	install -m 644 target/release/duplikatd.service $(DESTDIR)$(prefix)/lib/systemd/system/
	install -m 644 duplikatd/duplikatd.socket $(DESTDIR)$(prefix)/lib/systemd/system/

uninstall:
	rm $(DESTDIR)$(prefix)/bin/duplikat
	rmdir $(DESTDIR)$(prefix)/bin
	rm $(DESTDIR)$(prefix)/sbin/duplikatd
	rmdir $(DESTDIR)$(prefix)/sbin
	test -L /etc/systemd/system/sockets.target.wants/duplikatd.socket && \
		rm /etc/systemd/system/sockets.target.wants/duplikatd.socket
	test -L /etc/systemd/system/multi-user.target.wants/duplikatd.service && \
		rm /etc/systemd/system/multi-user.target.wants/duplikatd.service
	rm $(DESTDIR)$(prefix)/lib/systemd/system/duplikatd.service
	rm $(DESTDIR)$(prefix)/lib/systemd/system/duplikatd.socket
	rmdir $(DESTDIR)$(prefix)/lib/systemd/system/
