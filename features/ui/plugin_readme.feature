Feature: Plugin setup guide

  A plugin's README.md is its setup guide (which broker query to build, which
  fields, where the token comes from). The Plugins view shows it behind a
  Setup toggle so nobody has to find the file in the repo.

  Scenario: The Setup panel shows the plugin's README
    Given the app is running
    And a plugin "readme-demo" with a README is installed in the vault
    When I switch to the plugins view
    And I open the setup panel for plugin "readme-demo"
    Then the setup panel for plugin "readme-demo" shows "Setting up readme-demo"
    And the setup panel for plugin "readme-demo" opens its links outside the app
