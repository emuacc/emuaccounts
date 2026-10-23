parser grammar Prices;

options { tokenVocab=LedgerLexer; }

pricesFile
  : priceRecord* EOF
  ;

priceRecord
  : blankLine
  | directive
  | priceDirective
  ;

priceDirective
  : PRICE_DIRECTIVE WS DATETIME WS commodity WS amount WS commodity WS* lineEnd
  ;

directive
  : WS? SEMI (WS | anyToken)* lineEnd
  ;

blankLine
  : WS? NEWLINE
  ;

commodityInput
  : commodity EOF
  ;

amountInput
  : amount EOF
  ;

commodity
  : commodityHead commodityTail*
  ;

commodityHead
  : wordToken
  | NUMBER
  ;

commodityTail
  : commodityHead
  | DOT
  | MINUS
  | COLON
  ;

amount
  : (PLUS | MINUS)? (NUMBER | GROUPED_NUMBER)
  ;

wordToken
  : ALNUM
  | ACCOUNT
  | OPENING
  | NAME
  | PRICE_DIRECTIVE
  ;

anyToken
  : wordToken
  | NUMBER
  | GROUPED_NUMBER
  | DATETIME
  | QUOTED
  | DOUBLE_LBRACE
  | DOUBLE_RBRACE
  | ATAT
  | LBRACE
  | RBRACE
  | AT
  | SEMI
  | COMMA
  | COLON
  | STAR
  | BANG
  | PLUS
  | MINUS
  | DOT
  | OTHER
  ;

lineEnd
  : NEWLINE
  | EOF
  ;
