# Duplikat

Duplikat will be a Gnome backup application (that should also run on Mac OS at
least). It will use restic to perform the actual backups. To allow for full
system backups Duplikat wil be split in two parts: a privileged system daemon,
which actually runs restic, and an unpriviledged Gtk4-based interface that
allows for user-friendly configuration of backups and restoring.

The communication between server and application will be through regular HTTP
using json as the main data exchange format. This should allow for easily
developing other frontends, including web-facing ones.
