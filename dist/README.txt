SMART ORGANIZER 0.1.0
=====================

Smart Organizer tidies a folder such as Downloads or the Desktop. It shows the
destination of every file before moving anything, moves only what you approve,
and records every batch so you can undo it.

Windows 10/11, 64-bit. No installer and no administrator rights are required.

FILES
-----
Smart Organizer.exe       Desktop app and hidden watcher.
smart-organizer-cli.exe   Command-line build.
SHA256SUMS.txt            SHA-256 of every file in this release.
LEEME.txt                 Instrucciones en español.
README.txt                This file.

START IN THREE STEPS
--------------------
1. Open Smart Organizer.exe and press "Add folder". Pick Downloads or Desktop,
   or drop a folder onto the window.
2. Open "Preview" and check the source, destination and size of every file.
   Uncheck anything you want to leave alone.
3. Press "Organize now". Open "History" to undo the batch if needed.

SAFETY
------
The normal action is move, not delete. Files that match no group stay put.
The Recycle Bin is used only by an explicit Trash rule or when you choose
duplicates and press "Send to Recycle Bin". Duplicate cleanup verifies that the
copy you keep still exists and that its full hash matches before moving
anything. Hidden files, temporary download fragments, symlinks and protected
system folders are skipped.

If an organization pass is interrupted, the app checks every pending journal
entry on startup. When the source and destination both exist, it compares size
and BLAKE3, marks the entry "needs review", deletes nothing and warns in
History. A failed watcher pass is caught and logged; the GUI can retry it with
short waits, up to five attempts in ten minutes.

DATA
----
Settings and history are stored in:

  %APPDATA%\smart-organizer\config.toml
  %APPDATA%\smart-organizer\config.toml.bak
  %APPDATA%\smart-organizer\journal.jsonl
  %APPDATA%\smart-organizer\error.log

Open that folder from Settings > Data.

Saving writes config.toml.tmp, syncs it and replaces config.toml atomically.
The previous file is copied to config.toml.bak; a pre-migration
config.toml.v{version}.bak is created once and never overwritten. If the
replace fails, the old config.toml stays valid. Temporary files end in .tmp.

Portable mode: create a folder named "portable" next to Smart Organizer.exe
before starting it. The app then writes config.toml and journal.jsonl beside
the executable instead. Remove the folder to return to %APPDATA%.

Backup: quit the app and copy config.toml and journal.jsonl. Reset: Settings >
Data > Reset settings, or delete the data folder after backing up. Deleting the
journal removes the ability to undo old batches. The app never deletes your
documents.

SMARTSCREEN AND SHA-256
-----------------------
This release is not code-signed. Windows SmartScreen may show "Windows
protected your PC"; this is expected for a new unsigned executable. Check the
hash first, then use More info > Run anyway only if you trust the file. Do not
disable SmartScreen globally. A matching SHA-256 proves the file is the one whose
hash the author published and that it was not corrupted in transit; it does not
prove that the program is safe or that the author is trustworthy.

SHA-256 of Smart Organizer.exe:
  77CF5BB77DBAC252056A7E793EF7C4E366025C688D6068497A9CF1EA1D4B767E

In PowerShell:

  Get-FileHash ".\Smart Organizer.exe" -Algorithm SHA256
  Get-Content .\SHA256SUMS.txt

WATCHING AND TRAY
-----------------
Watching runs in a separate hidden process. Closing the window while watching
leaves it in the tray. If watching crashes, the GUI retries after 1, 2, 4, 8 and
16 seconds, at most five attempts in ten minutes, then shows a warning and stops
retrying. Use "Restart watching" in Settings > Watching, Home, the simple view,
the status bar or the tray. If the GUI cannot open, run Smart Organizer.exe
--watch to keep watching; the reason is written to error.log.

PRIVACY
-------
The app reads only the folders you add. It has no telemetry, account, ads,
updater or network client. Hash reads and metadata stay on this computer.

LICENSE AND SUPPORT
-------------------
The package metadata declares MIT.

TODO: add a LICENSE file and the copyright holder.
TODO: add the public repository URL.
TODO: add a support contact or issue tracker.

Until then the files are provided as-is, without a support commitment.
