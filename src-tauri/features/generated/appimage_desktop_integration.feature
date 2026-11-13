Feature: Run from an AppImage, EmuAcc installs its desktop entry, so docks show the emu

  On Wayland a GTK3 window sends no icon. Hyprland, Waybar and other docks
  find it through the installed desktop entry named after the window's
  app_id, which is the binary's name, emuaccounts. The .deb installs one; a
  downloaded AppImage installs nothing, so its window showed the generic cog
  (TASK-140). Run from an AppImage, EmuAcc installs the entry and the icons
  the AppImage carries into the user's data folder before its window opens,
  and rewrites only the files that differ. Paths below are under the user's
  home folder.

  Background:
    Given an AppImage of "EmuAcc" at "Downloads/EmuAcc_0.1.4_amd64.AppImage" carrying the emu icons:
      | 32x32   |
      | 128x128 |
      | 512x512 |

  Scenario: The first launch installs the desktop entry and the icons
    When EmuAcc starts
    Then the desktop entry ".local/share/applications/emuaccounts.desktop" reads:
      | Type           | Application                                    |
      | Name           | EmuAcc                                         |
      | Exec           | "{home}/Downloads/EmuAcc_0.1.4_amd64.AppImage" |
      | TryExec        | {home}/Downloads/EmuAcc_0.1.4_amd64.AppImage   |
      | Icon           | emuaccounts                                    |
      | StartupWMClass | emuaccounts                                    |
      | Terminal       | false                                          |
    And these icons are the AppImage's:
      | .local/share/icons/hicolor/32x32/apps/emuaccounts.png   |
      | .local/share/icons/hicolor/128x128/apps/emuaccounts.png |
      | .local/share/icons/hicolor/512x512/apps/emuaccounts.png |

  Scenario: Relaunching the same AppImage writes nothing
    Given EmuAcc started
    When EmuAcc starts
    Then nothing is written

  Scenario: A moved AppImage rewrites only the desktop entry
    Given EmuAcc started
    When the AppImage moves to "Applications/Emu Acc.AppImage"
    And EmuAcc starts
    Then only these files are written:
      | .local/share/applications/emuaccounts.desktop |
    And the desktop entry ".local/share/applications/emuaccounts.desktop" runs "{home}/Applications/Emu Acc.AppImage"

  Scenario: A changed AppImage rewrites only the icons that changed
    Given EmuAcc started
    When the AppImage's "128x128" icon changes
    And EmuAcc starts
    Then only these files are written:
      | .local/share/icons/hicolor/128x128/apps/emuaccounts.png |

  Scenario: Installing an icon retires a stale icon-theme cache
    # GTK ignores a hicolor icon-theme.cache older than its folder; one built
    # before the emu was installed would otherwise hide it.
    Given ".local/share/icons/hicolor" has an icon-theme cache from an hour ago
    When EmuAcc starts
    Then ".local/share/icons/hicolor" is newer than its icon-theme cache

  Scenario: XDG_DATA_HOME chooses the user's data folder
    Given XDG_DATA_HOME is "{home}/xdg-data"
    When EmuAcc starts
    Then the desktop entry "xdg-data/applications/emuaccounts.desktop" runs "{home}/Downloads/EmuAcc_0.1.4_amd64.AppImage"

  Scenario: The .deb and dev builds install nothing
    Given APPIMAGE is not set
    When EmuAcc starts
    Then it is not an AppImage launch
    And ".local/share" does not exist

  Scenario: An APPIMAGE inherited from another AppImage installs nothing
    # A terminal started from an AppImage passes its APPIMAGE and APPDIR on
    # to a .deb or dev EmuAcc started from it.
    Given EmuAcc's binary is outside the AppImage's mount
    When EmuAcc starts
    Then it is not an AppImage launch
    And ".local/share" does not exist

  Scenario: The Linux window icon is the 256px emu
    # Tauri makes the first PNG in bundle.icon the window icon on Linux; X11
    # docks show that icon when no desktop entry matches the window.
    Then the first PNG in the bundle's icons is 256x256
