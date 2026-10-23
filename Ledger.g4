parser grammar Ledger;

options { tokenVocab=LedgerLexer; }

/*
 * Authoritative ledger record grammar.
 *
 * This grammar intentionally accepts opaque metadata and annotation fields.
 * The Rust semantic adapter derives balances, fees, source-display fields,
 * and domain diagnostics from these parse-tree records.
 *
 * The handwritten runtime parser (src-tauri/src/ledger_parser.rs) is held
 * to this grammar by differential parity tests
 * (src-tauri/src/ledger_parser/grammar_parity.rs); change the grammar
 * first, regenerate, then the parser. See decision-003.
 */

file
  : record* EOF
  ;

record
  : blankLine
  | directive
  | accountDeclaration
  | transaction
  ;

transaction
  : headerLine postingLine+ blankLine*
  ;

// Every transaction carries a meta comment (`; txn:<id>, ...`): rules, trade
// links and the UI address transactions by the ids in it.
headerLine
  : DATETIME WS status? WS* headerField (WS headerField)* WS* metaComment lineEnd
  ;

status
  : STAR
  | BANG
  ;

headerField
  : QUOTED
  | bareValue
  ;

postingLine
  : WS account WS amount WS commodity postingComponent* lineEnd
  ;

postingComponent
  : WS
  | lotCostSpec
  | priceSpec
  | metaComment
  | looseToken
  ;

accountDeclaration
  : accountHeader accountDetailLine* blankLine*
  ;

accountHeader
  : ACCOUNT WS account (WS commodity)? WS* metaComment? lineEnd
  ;

accountDetailLine
  : WS (openingDetail | nameDetail) WS* metaComment? lineEnd
  ;

openingDetail
  : OPENING WS amount (WS commodity)?
  ;

nameDetail
  : NAME WS nameValue
  ;

nameValue
  : headerField (WS headerField)*
  ;

directive
  : WS? metaComment lineEnd
  ;

blankLine
  : WS? NEWLINE
  ;

metaComment
  : SEMI (WS | anyToken)*
  ;

/* ----- Accounts, commodities, and amounts ----- */

accountInput
  : account EOF
  ;

commodityInput
  : commodity EOF
  ;

amountInput
  : amount EOF
  ;

account
  : accountSegment COLON accountSegment (COLON accountSegment)*
  ;

accountSegment
  : accountPiece+
  ;

// Like a commodity, an account never contains a date-shaped piece: the lexer
// makes YYYY-MM-DD a DATETIME wherever a token starts.
accountPiece
  : wordToken
  | NUMBER
  | DOT
  | MINUS
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

/* ----- Cost and price annotations ----- */

lotCostSpec
  : LBRACE costBody RBRACE
  | DOUBLE_LBRACE costBody DOUBLE_RBRACE
  ;

costBody
  : WS* costField? (WS* COMMA WS* costField)* WS*
  ;

costField
  : costValue (WS costValue)*
  ;

costValue
  : (QUOTED | barePiece)+
  ;

priceSpec
  : (AT | ATAT) WS* amount WS commodity
  ;

/* ----- Opaque values and recovery ----- */

bareValue
  : barePiece+
  ;

barePiece
  : wordToken
  | NUMBER
  | GROUPED_NUMBER
  | DATETIME
  | DOT
  | MINUS
  | PLUS
  | COLON
  | OTHER
  ;

wordToken
  : ALNUM
  | ACCOUNT
  | OPENING
  | NAME
  | PRICE_DIRECTIVE
  ;

looseToken
  : QUOTED
  | COMMA
  | bareValue
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
