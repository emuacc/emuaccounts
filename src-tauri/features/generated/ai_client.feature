@ai_client
Feature: OpenAI-compatible AI client

  The app's AI features talk to a single OpenAI-compatible chat-completions
  endpoint configured by API URL + API key + model in Global Settings
  (config.json). Calls may carry tool definitions; the client runs the
  tool-call loop until the model returns a final answer.

  Scenario: Unconfigured AI points the user at Global Settings
    Given an app config with no AI provider settings
    When I request an AI completion for prompt "hello"
    Then the AI request should fail with an error mentioning "Global Settings"

  Scenario: Direct answer from an OpenAI-compatible server
    Given a stub AI server that replies with content "hello from stub"
    When I request an AI completion for prompt "hello"
    Then the AI answer should be "hello from stub"

  Scenario: Tool call round trip
    Given a stub AI server that requests the "echo_upper" tool for text "abc" and then replies with content "done"
    When I request an AI completion for prompt "hello" with an "echo_upper" tool
    Then the AI answer should be "done"
    And the stub server should have received a tool result "ABC" for tool call "call_1"
    And the AI step log should mention "echo_upper"
