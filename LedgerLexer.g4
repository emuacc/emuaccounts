lexer grammar LedgerLexer;

// Shared token vocabulary for Ledger.g4 and Prices.g4. Whitespace remains on
// the default channel because indentation and field boundaries are syntax.

ACCOUNT: 'account';
OPENING: 'opening';
NAME: 'name';
PRICE_DIRECTIVE: 'P';

DOUBLE_LBRACE: '{{';
DOUBLE_RBRACE: '}}';
ATAT: '@@';
LBRACE: '{';
RBRACE: '}';
AT: '@';
SEMI: ';';
COMMA: ',';
COLON: ':';
STAR: '*';
BANG: '!';
PLUS: '+';
MINUS: '-';
DOT: '.';

DATETIME
  : DIGIT DIGIT DIGIT DIGIT '-' DIGIT DIGIT '-' DIGIT DIGIT
    ( ('T' | ' ')
      DIGIT DIGIT ':' DIGIT DIGIT ':' DIGIT DIGIT
      ('.' DIGIT DIGIT? DIGIT? DIGIT? DIGIT? DIGIT?)?
      ( 'Z' | (('+' | '-') DIGIT DIGIT ':' DIGIT DIGIT) )?
    )?
  ;

GROUPED_NUMBER
  : DIGIT DIGIT? DIGIT? (',' DIGIT DIGIT DIGIT)+ ('.' DIGIT+)?
  ;

NUMBER
  : DIGIT+ ('.' DIGIT+)?
  ;

QUOTED
  : '"' ( '\\"' | '\\\\' | ~["\r\n] )* '"'
  ;

ALNUM
  : [\p{L}\p{N}_]+
  ;

WS
  : [ \t]+
  ;

NEWLINE
  : '\r'? '\n'
  ;

// Preserve otherwise-opaque header, metadata, and annotation characters.
// Specific tokens above win equal-length matches; OTHER guarantees recovery
// never loses source text merely because metadata uses new punctuation.
OTHER
  : .
  ;

fragment DIGIT
  : [0-9]
  ;
