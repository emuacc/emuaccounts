// Generated from Ledger.g4 by ANTLR 4.13.2
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(nonstandard_style)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_braces)]
#![allow(unused_parens)]
use antlr4rust::PredictionContextCache;
use antlr4rust::parser::{Parser, BaseParser, ParserRecog, ParserNodeType};
use antlr4rust::token_stream::TokenStream;
use antlr4rust::TokenSource;
use antlr4rust::parser_atn_simulator::ParserATNSimulator;
use antlr4rust::errors::*;
use antlr4rust::rule_context::{BaseRuleContext, CustomRuleContext, RuleContext};
use antlr4rust::recognizer::{Recognizer,Actions};
use antlr4rust::atn_deserializer::ATNDeserializer;
use antlr4rust::dfa::DFA;
use antlr4rust::atn::{ATN, INVALID_ALT};
use antlr4rust::error_strategy::{ErrorStrategy, DefaultErrorStrategy};
use antlr4rust::parser_rule_context::{BaseParserRuleContext, ParserRuleContext,cast,cast_mut};
use antlr4rust::tree::*;
use antlr4rust::token::{TOKEN_EOF,OwningToken,Token};
use antlr4rust::int_stream::EOF;
use antlr4rust::vocabulary::{Vocabulary,VocabularyImpl};
use antlr4rust::token_factory::{CommonTokenFactory,TokenFactory, TokenAware};
use super::ledgervisitor::*;

use antlr4rust::lazy_static;
use antlr4rust::{TidAble,TidExt};

use std::marker::PhantomData;
use std::sync::Arc;
use std::rc::Rc;
use std::convert::TryFrom;
use std::cell::RefCell;
use std::ops::{DerefMut, Deref};
use std::borrow::{Borrow,BorrowMut};
use std::any::{Any,TypeId};

		pub const Ledger_ACCOUNT:i32=1; 
		pub const Ledger_OPENING:i32=2; 
		pub const Ledger_NAME:i32=3; 
		pub const Ledger_PRICE_DIRECTIVE:i32=4; 
		pub const Ledger_DOUBLE_LBRACE:i32=5; 
		pub const Ledger_DOUBLE_RBRACE:i32=6; 
		pub const Ledger_ATAT:i32=7; 
		pub const Ledger_LBRACE:i32=8; 
		pub const Ledger_RBRACE:i32=9; 
		pub const Ledger_AT:i32=10; 
		pub const Ledger_SEMI:i32=11; 
		pub const Ledger_COMMA:i32=12; 
		pub const Ledger_COLON:i32=13; 
		pub const Ledger_STAR:i32=14; 
		pub const Ledger_BANG:i32=15; 
		pub const Ledger_PLUS:i32=16; 
		pub const Ledger_MINUS:i32=17; 
		pub const Ledger_DOT:i32=18; 
		pub const Ledger_DATETIME:i32=19; 
		pub const Ledger_GROUPED_NUMBER:i32=20; 
		pub const Ledger_NUMBER:i32=21; 
		pub const Ledger_QUOTED:i32=22; 
		pub const Ledger_ALNUM:i32=23; 
		pub const Ledger_WS:i32=24; 
		pub const Ledger_NEWLINE:i32=25; 
		pub const Ledger_OTHER:i32=26;
	pub const Ledger_EOF:i32=EOF;
	pub const RULE_file:usize = 0; 
	pub const RULE_record:usize = 1; 
	pub const RULE_transaction:usize = 2; 
	pub const RULE_headerLine:usize = 3; 
	pub const RULE_status:usize = 4; 
	pub const RULE_headerField:usize = 5; 
	pub const RULE_postingLine:usize = 6; 
	pub const RULE_postingComponent:usize = 7; 
	pub const RULE_accountDeclaration:usize = 8; 
	pub const RULE_accountHeader:usize = 9; 
	pub const RULE_accountDetailLine:usize = 10; 
	pub const RULE_openingDetail:usize = 11; 
	pub const RULE_nameDetail:usize = 12; 
	pub const RULE_nameValue:usize = 13; 
	pub const RULE_directive:usize = 14; 
	pub const RULE_blankLine:usize = 15; 
	pub const RULE_metaComment:usize = 16; 
	pub const RULE_accountInput:usize = 17; 
	pub const RULE_commodityInput:usize = 18; 
	pub const RULE_amountInput:usize = 19; 
	pub const RULE_account:usize = 20; 
	pub const RULE_accountSegment:usize = 21; 
	pub const RULE_accountPiece:usize = 22; 
	pub const RULE_commodity:usize = 23; 
	pub const RULE_commodityHead:usize = 24; 
	pub const RULE_commodityTail:usize = 25; 
	pub const RULE_amount:usize = 26; 
	pub const RULE_lotCostSpec:usize = 27; 
	pub const RULE_costBody:usize = 28; 
	pub const RULE_costField:usize = 29; 
	pub const RULE_costValue:usize = 30; 
	pub const RULE_priceSpec:usize = 31; 
	pub const RULE_bareValue:usize = 32; 
	pub const RULE_barePiece:usize = 33; 
	pub const RULE_wordToken:usize = 34; 
	pub const RULE_looseToken:usize = 35; 
	pub const RULE_anyToken:usize = 36; 
	pub const RULE_lineEnd:usize = 37;
	pub const ruleNames: [&'static str; 38] =  [
		"file", "record", "transaction", "headerLine", "status", "headerField", 
		"postingLine", "postingComponent", "accountDeclaration", "accountHeader", 
		"accountDetailLine", "openingDetail", "nameDetail", "nameValue", "directive", 
		"blankLine", "metaComment", "accountInput", "commodityInput", "amountInput", 
		"account", "accountSegment", "accountPiece", "commodity", "commodityHead", 
		"commodityTail", "amount", "lotCostSpec", "costBody", "costField", "costValue", 
		"priceSpec", "bareValue", "barePiece", "wordToken", "looseToken", "anyToken", 
		"lineEnd"
	];


	pub const _LITERAL_NAMES: [Option<&'static str>;19] = [
		None, Some("'account'"), Some("'opening'"), Some("'name'"), Some("'P'"), 
		Some("'{{'"), Some("'}}'"), Some("'@@'"), Some("'{'"), Some("'}'"), Some("'@'"), 
		Some("';'"), Some("','"), Some("':'"), Some("'*'"), Some("'!'"), Some("'+'"), 
		Some("'-'"), Some("'.'")
	];
	pub const _SYMBOLIC_NAMES: [Option<&'static str>;27]  = [
		None, Some("ACCOUNT"), Some("OPENING"), Some("NAME"), Some("PRICE_DIRECTIVE"), 
		Some("DOUBLE_LBRACE"), Some("DOUBLE_RBRACE"), Some("ATAT"), Some("LBRACE"), 
		Some("RBRACE"), Some("AT"), Some("SEMI"), Some("COMMA"), Some("COLON"), 
		Some("STAR"), Some("BANG"), Some("PLUS"), Some("MINUS"), Some("DOT"), 
		Some("DATETIME"), Some("GROUPED_NUMBER"), Some("NUMBER"), Some("QUOTED"), 
		Some("ALNUM"), Some("WS"), Some("NEWLINE"), Some("OTHER")
	];
	lazy_static!{
	    static ref _shared_context_cache: Arc<PredictionContextCache> = Arc::new(PredictionContextCache::new());
		static ref VOCABULARY: Box<dyn Vocabulary> = Box::new(VocabularyImpl::new(_LITERAL_NAMES.iter(), _SYMBOLIC_NAMES.iter(), None));
	}


// antlr4rust runtime compatibility for visitor-only generation.
pub trait LedgerNoOpListener<'input>: ParseTreeListener<'input,LedgerContextType> {}
antlr4rust::coerce_from!{ 'input : LedgerNoOpListener<'input> }
antlr4rust::tid! { impl<'input> TidAble<'input> for dyn LedgerNoOpListener<'input> + 'input }

type BaseParserType<'input, I> =
	BaseParser<'input,LedgerExt<'input>, I, LedgerContextType, dyn LedgerNoOpListener<'input> + 'input >;

type TokenType<'input> = <LocalTokenFactory<'input> as TokenFactory<'input>>::Tok;
pub type LocalTokenFactory<'input> = CommonTokenFactory;

pub type LedgerTreeWalker<'input,'a> =
	ParseTreeWalker<'input, 'a, LedgerContextType, dyn LedgerNoOpListener<'input> + 'a >;

/// Parser for Ledger grammar
pub struct Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	base:BaseParserType<'input,I>,
	interpreter:Arc<ParserATNSimulator>,
	_shared_context_cache: Box<PredictionContextCache>,
    pub err_handler: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >,
}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn set_error_strategy(&mut self, strategy: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >) {
        self.err_handler = strategy
    }

    pub fn with_strategy(input: I, strategy: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >) -> Self {
		antlr4rust::recognizer::check_version("0","5");
		let interpreter = Arc::new(ParserATNSimulator::new(
			_ATN.clone(),
			_decision_to_DFA.clone(),
			_shared_context_cache.clone(),
		));
		Self {
			base: BaseParser::new_base_parser(
				input,
				Arc::clone(&interpreter),
				LedgerExt{
					_pd: Default::default(),
				}
			),
			interpreter,
            _shared_context_cache: Box::new(PredictionContextCache::new()),
            err_handler: strategy,
        }
    }

}

type DynStrategy<'input,I> = Box<dyn ErrorStrategy<'input,BaseParserType<'input,I>> + 'input>;

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn with_dyn_strategy(input: I) -> Self{
    	Self::with_strategy(input,Box::new(DefaultErrorStrategy::new()))
    }
}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn new(input: I) -> Self{
    	Self::with_strategy(input,Box::new(DefaultErrorStrategy::new()))
    }
}

/// Trait for monomorphized trait object that corresponds to the nodes of parse tree generated for Ledger
pub trait LedgerContext<'input>:
	for<'x> Visitable<dyn LedgerVisitor<'input> + 'x > + 
	for<'x> Listenable<dyn LedgerNoOpListener<'input> + 'x > + 
	ParserRuleContext<'input, TF=LocalTokenFactory<'input>, Ctx=LedgerContextType>
{}

antlr4rust::coerce_from!{ 'input : LedgerContext<'input> }

impl<'input, 'x, T> VisitableDyn<T> for dyn LedgerContext<'input> + 'input
where
    T: LedgerVisitor<'input> + 'x,
{
    fn accept_dyn(&self, visitor: &mut T) {
        self.accept(visitor as &mut (dyn LedgerVisitor<'input> + 'x))
    }
}

impl<'input> LedgerContext<'input> for TerminalNode<'input,LedgerContextType> {}
impl<'input> LedgerContext<'input> for ErrorNode<'input,LedgerContextType> {}

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn LedgerContext<'input> + 'input }


pub struct LedgerContextType;
antlr4rust::tid!{LedgerContextType}

impl<'input> ParserNodeType<'input> for LedgerContextType{
	type TF = LocalTokenFactory<'input>;
	type Type = dyn LedgerContext<'input> + 'input;
}

impl<'input, I> Deref for Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    type Target = BaseParserType<'input,I>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'input, I> DerefMut for Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct LedgerExt<'input>{
	_pd: PhantomData<&'input str>,
}

impl<'input> LedgerExt<'input>{
}
antlr4rust::tid! { LedgerExt<'a> }

impl<'input> TokenAware<'input> for LedgerExt<'input>{
	type TF = LocalTokenFactory<'input>;
}

impl<'input,I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>> ParserRecog<'input, BaseParserType<'input,I>> for LedgerExt<'input>{}

impl<'input,I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>> Actions<'input, BaseParserType<'input,I>> for LedgerExt<'input>{
	fn get_grammar_file_name(&self) -> & str{ "Ledger.g4"}

   	fn get_rule_names(&self) -> &[& str] {&ruleNames}

   	fn get_vocabulary(&self) -> &dyn Vocabulary { &**VOCABULARY }
}
//------------------- file ----------------
pub type FileContextAll<'input> = FileContext<'input>;


pub type FileContext<'input> = BaseParserRuleContext<'input,FileContextExt<'input>>;

#[derive(Clone)]
pub struct FileContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for FileContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for FileContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for FileContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_file(self);
	}
}

impl<'input> CustomRuleContext<'input> for FileContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_file }
	//fn type_rule_index() -> usize where Self: Sized { RULE_file }
}
antlr4rust::tid!{FileContextExt<'a>}

impl<'input> FileContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<FileContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,FileContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait FileContextAttrs<'input>: LedgerContext<'input> + BorrowMut<FileContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_EOF, 0)
}
fn record_all(&self) ->  Vec<Rc<RecordContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn record(&self, i: usize) -> Option<Rc<RecordContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> FileContextAttrs<'input> for FileContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn file(&mut self,)
	-> Result<Rc<FileContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = FileContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 0, RULE_file);
        let mut _localctx: Rc<FileContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(79);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 50857986) != 0) {
				{
				{
				/*InvokeRule record*/
				recog.base.set_state(76);
				recog.record()?;

				}
				}
				recog.base.set_state(81);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(82);
			recog.base.match_token(Ledger_EOF,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- record ----------------
pub type RecordContextAll<'input> = RecordContext<'input>;


pub type RecordContext<'input> = BaseParserRuleContext<'input,RecordContextExt<'input>>;

#[derive(Clone)]
pub struct RecordContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for RecordContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for RecordContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for RecordContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_record(self);
	}
}

impl<'input> CustomRuleContext<'input> for RecordContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_record }
	//fn type_rule_index() -> usize where Self: Sized { RULE_record }
}
antlr4rust::tid!{RecordContextExt<'a>}

impl<'input> RecordContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<RecordContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,RecordContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait RecordContextAttrs<'input>: LedgerContext<'input> + BorrowMut<RecordContextExt<'input>>{

fn blankLine(&self) -> Option<Rc<BlankLineContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn directive(&self) -> Option<Rc<DirectiveContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn accountDeclaration(&self) -> Option<Rc<AccountDeclarationContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn transaction(&self) -> Option<Rc<TransactionContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> RecordContextAttrs<'input> for RecordContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn record(&mut self,)
	-> Result<Rc<RecordContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = RecordContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 2, RULE_record);
        let mut _localctx: Rc<RecordContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(88);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(1,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule blankLine*/
					recog.base.set_state(84);
					recog.blankLine()?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule directive*/
					recog.base.set_state(85);
					recog.directive()?;

					}
				}
			,
				3 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule accountDeclaration*/
					recog.base.set_state(86);
					recog.accountDeclaration()?;

					}
				}
			,
				4 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule transaction*/
					recog.base.set_state(87);
					recog.transaction()?;

					}
				}

				_ => {}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- transaction ----------------
pub type TransactionContextAll<'input> = TransactionContext<'input>;


pub type TransactionContext<'input> = BaseParserRuleContext<'input,TransactionContextExt<'input>>;

#[derive(Clone)]
pub struct TransactionContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for TransactionContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for TransactionContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for TransactionContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_transaction(self);
	}
}

impl<'input> CustomRuleContext<'input> for TransactionContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_transaction }
	//fn type_rule_index() -> usize where Self: Sized { RULE_transaction }
}
antlr4rust::tid!{TransactionContextExt<'a>}

impl<'input> TransactionContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<TransactionContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,TransactionContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait TransactionContextAttrs<'input>: LedgerContext<'input> + BorrowMut<TransactionContextExt<'input>>{

fn headerLine(&self) -> Option<Rc<HeaderLineContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn postingLine_all(&self) ->  Vec<Rc<PostingLineContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn postingLine(&self, i: usize) -> Option<Rc<PostingLineContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn blankLine_all(&self) ->  Vec<Rc<BlankLineContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn blankLine(&self, i: usize) -> Option<Rc<BlankLineContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> TransactionContextAttrs<'input> for TransactionContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn transaction(&mut self,)
	-> Result<Rc<TransactionContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = TransactionContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 4, RULE_transaction);
        let mut _localctx: Rc<TransactionContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule headerLine*/
			recog.base.set_state(90);
			recog.headerLine()?;

			recog.base.set_state(92); 
			recog.err_handler.sync(&mut recog.base)?;
			_alt = 1;
			loop {
				match _alt {
				    x if x == 1=>
					{
					{
					/*InvokeRule postingLine*/
					recog.base.set_state(91);
					recog.postingLine()?;

					}
					}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
				}
				recog.base.set_state(94); 
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(2,&mut recog.base)?;
				if _alt==2 || _alt==INVALID_ALT { break }
			}
			recog.base.set_state(99);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(3,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					/*InvokeRule blankLine*/
					recog.base.set_state(96);
					recog.blankLine()?;

					}
					} 
				}
				recog.base.set_state(101);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(3,&mut recog.base)?;
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- headerLine ----------------
pub type HeaderLineContextAll<'input> = HeaderLineContext<'input>;


pub type HeaderLineContext<'input> = BaseParserRuleContext<'input,HeaderLineContextExt<'input>>;

#[derive(Clone)]
pub struct HeaderLineContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for HeaderLineContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for HeaderLineContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for HeaderLineContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_headerLine(self);
	}
}

impl<'input> CustomRuleContext<'input> for HeaderLineContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_headerLine }
	//fn type_rule_index() -> usize where Self: Sized { RULE_headerLine }
}
antlr4rust::tid!{HeaderLineContextExt<'a>}

impl<'input> HeaderLineContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<HeaderLineContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,HeaderLineContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait HeaderLineContextAttrs<'input>: LedgerContext<'input> + BorrowMut<HeaderLineContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token DATETIME
/// Returns `None` if there is no child corresponding to token DATETIME
fn DATETIME(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DATETIME, 0)
}
/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, i)
}
fn headerField_all(&self) ->  Vec<Rc<HeaderFieldContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn headerField(&self, i: usize) -> Option<Rc<HeaderFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn metaComment(&self) -> Option<Rc<MetaCommentContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn status(&self) -> Option<Rc<StatusContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> HeaderLineContextAttrs<'input> for HeaderLineContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn headerLine(&mut self,)
	-> Result<Rc<HeaderLineContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = HeaderLineContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 6, RULE_headerLine);
        let mut _localctx: Rc<HeaderLineContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(102);
			recog.base.match_token(Ledger_DATETIME,&mut recog.err_handler)?;

			recog.base.set_state(103);
			recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

			recog.base.set_state(105);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Ledger_STAR || _la==Ledger_BANG {
				{
				/*InvokeRule status*/
				recog.base.set_state(104);
				recog.status()?;

				}
			}

			recog.base.set_state(110);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Ledger_WS {
				{
				{
				recog.base.set_state(107);
				recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(112);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule headerField*/
			recog.base.set_state(113);
			recog.headerField()?;

			recog.base.set_state(118);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(6,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(114);
					recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

					/*InvokeRule headerField*/
					recog.base.set_state(115);
					recog.headerField()?;

					}
					} 
				}
				recog.base.set_state(120);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(6,&mut recog.base)?;
			}
			recog.base.set_state(124);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Ledger_WS {
				{
				{
				recog.base.set_state(121);
				recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(126);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule metaComment*/
			recog.base.set_state(127);
			recog.metaComment()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(128);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- status ----------------
pub type StatusContextAll<'input> = StatusContext<'input>;


pub type StatusContext<'input> = BaseParserRuleContext<'input,StatusContextExt<'input>>;

#[derive(Clone)]
pub struct StatusContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for StatusContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for StatusContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for StatusContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_status(self);
	}
}

impl<'input> CustomRuleContext<'input> for StatusContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_status }
	//fn type_rule_index() -> usize where Self: Sized { RULE_status }
}
antlr4rust::tid!{StatusContextExt<'a>}

impl<'input> StatusContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<StatusContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,StatusContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait StatusContextAttrs<'input>: LedgerContext<'input> + BorrowMut<StatusContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token STAR
/// Returns `None` if there is no child corresponding to token STAR
fn STAR(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_STAR, 0)
}
/// Retrieves first TerminalNode corresponding to token BANG
/// Returns `None` if there is no child corresponding to token BANG
fn BANG(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_BANG, 0)
}

}

impl<'input> StatusContextAttrs<'input> for StatusContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn status(&mut self,)
	-> Result<Rc<StatusContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = StatusContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 8, RULE_status);
        let mut _localctx: Rc<StatusContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(130);
			_la = recog.base.input.la(1);
			if { !(_la==Ledger_STAR || _la==Ledger_BANG) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- headerField ----------------
pub type HeaderFieldContextAll<'input> = HeaderFieldContext<'input>;


pub type HeaderFieldContext<'input> = BaseParserRuleContext<'input,HeaderFieldContextExt<'input>>;

#[derive(Clone)]
pub struct HeaderFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for HeaderFieldContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for HeaderFieldContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for HeaderFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_headerField(self);
	}
}

impl<'input> CustomRuleContext<'input> for HeaderFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_headerField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_headerField }
}
antlr4rust::tid!{HeaderFieldContextExt<'a>}

impl<'input> HeaderFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<HeaderFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,HeaderFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait HeaderFieldContextAttrs<'input>: LedgerContext<'input> + BorrowMut<HeaderFieldContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token QUOTED
/// Returns `None` if there is no child corresponding to token QUOTED
fn QUOTED(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_QUOTED, 0)
}
fn bareValue(&self) -> Option<Rc<BareValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> HeaderFieldContextAttrs<'input> for HeaderFieldContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn headerField(&mut self,)
	-> Result<Rc<HeaderFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = HeaderFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 10, RULE_headerField);
        let mut _localctx: Rc<HeaderFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(134);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Ledger_QUOTED 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(132);
					recog.base.match_token(Ledger_QUOTED,&mut recog.err_handler)?;

					}
				}

			Ledger_ACCOUNT |Ledger_OPENING |Ledger_NAME |Ledger_PRICE_DIRECTIVE |
			Ledger_COLON |Ledger_PLUS |Ledger_MINUS |Ledger_DOT |Ledger_DATETIME |
			Ledger_GROUPED_NUMBER |Ledger_NUMBER |Ledger_ALNUM |Ledger_OTHER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule bareValue*/
					recog.base.set_state(133);
					recog.bareValue()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- postingLine ----------------
pub type PostingLineContextAll<'input> = PostingLineContext<'input>;


pub type PostingLineContext<'input> = BaseParserRuleContext<'input,PostingLineContextExt<'input>>;

#[derive(Clone)]
pub struct PostingLineContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for PostingLineContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for PostingLineContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for PostingLineContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_postingLine(self);
	}
}

impl<'input> CustomRuleContext<'input> for PostingLineContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_postingLine }
	//fn type_rule_index() -> usize where Self: Sized { RULE_postingLine }
}
antlr4rust::tid!{PostingLineContextExt<'a>}

impl<'input> PostingLineContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<PostingLineContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PostingLineContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PostingLineContextAttrs<'input>: LedgerContext<'input> + BorrowMut<PostingLineContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, i)
}
fn account(&self) -> Option<Rc<AccountContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn amount(&self) -> Option<Rc<AmountContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn commodity(&self) -> Option<Rc<CommodityContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn postingComponent_all(&self) ->  Vec<Rc<PostingComponentContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn postingComponent(&self, i: usize) -> Option<Rc<PostingComponentContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> PostingLineContextAttrs<'input> for PostingLineContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn postingLine(&mut self,)
	-> Result<Rc<PostingLineContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PostingLineContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 12, RULE_postingLine);
        let mut _localctx: Rc<PostingLineContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(136);
			recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

			/*InvokeRule account*/
			recog.base.set_state(137);
			recog.account()?;

			recog.base.set_state(138);
			recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

			/*InvokeRule amount*/
			recog.base.set_state(139);
			recog.amount()?;

			recog.base.set_state(140);
			recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

			/*InvokeRule commodity*/
			recog.base.set_state(141);
			recog.commodity()?;

			recog.base.set_state(145);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 100613566) != 0) {
				{
				{
				/*InvokeRule postingComponent*/
				recog.base.set_state(142);
				recog.postingComponent()?;

				}
				}
				recog.base.set_state(147);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule lineEnd*/
			recog.base.set_state(148);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- postingComponent ----------------
pub type PostingComponentContextAll<'input> = PostingComponentContext<'input>;


pub type PostingComponentContext<'input> = BaseParserRuleContext<'input,PostingComponentContextExt<'input>>;

#[derive(Clone)]
pub struct PostingComponentContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for PostingComponentContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for PostingComponentContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for PostingComponentContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_postingComponent(self);
	}
}

impl<'input> CustomRuleContext<'input> for PostingComponentContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_postingComponent }
	//fn type_rule_index() -> usize where Self: Sized { RULE_postingComponent }
}
antlr4rust::tid!{PostingComponentContextExt<'a>}

impl<'input> PostingComponentContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<PostingComponentContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PostingComponentContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PostingComponentContextAttrs<'input>: LedgerContext<'input> + BorrowMut<PostingComponentContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token WS
/// Returns `None` if there is no child corresponding to token WS
fn WS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, 0)
}
fn lotCostSpec(&self) -> Option<Rc<LotCostSpecContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn priceSpec(&self) -> Option<Rc<PriceSpecContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn metaComment(&self) -> Option<Rc<MetaCommentContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn looseToken(&self) -> Option<Rc<LooseTokenContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> PostingComponentContextAttrs<'input> for PostingComponentContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn postingComponent(&mut self,)
	-> Result<Rc<PostingComponentContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PostingComponentContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 14, RULE_postingComponent);
        let mut _localctx: Rc<PostingComponentContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(155);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Ledger_WS 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(150);
					recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

					}
				}

			Ledger_DOUBLE_LBRACE |Ledger_LBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule lotCostSpec*/
					recog.base.set_state(151);
					recog.lotCostSpec()?;

					}
				}

			Ledger_ATAT |Ledger_AT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule priceSpec*/
					recog.base.set_state(152);
					recog.priceSpec()?;

					}
				}

			Ledger_SEMI 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					/*InvokeRule metaComment*/
					recog.base.set_state(153);
					recog.metaComment()?;

					}
				}

			Ledger_ACCOUNT |Ledger_OPENING |Ledger_NAME |Ledger_PRICE_DIRECTIVE |
			Ledger_COMMA |Ledger_COLON |Ledger_PLUS |Ledger_MINUS |Ledger_DOT |Ledger_DATETIME |
			Ledger_GROUPED_NUMBER |Ledger_NUMBER |Ledger_QUOTED |Ledger_ALNUM |Ledger_OTHER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					/*InvokeRule looseToken*/
					recog.base.set_state(154);
					recog.looseToken()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- accountDeclaration ----------------
pub type AccountDeclarationContextAll<'input> = AccountDeclarationContext<'input>;


pub type AccountDeclarationContext<'input> = BaseParserRuleContext<'input,AccountDeclarationContextExt<'input>>;

#[derive(Clone)]
pub struct AccountDeclarationContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for AccountDeclarationContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for AccountDeclarationContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for AccountDeclarationContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_accountDeclaration(self);
	}
}

impl<'input> CustomRuleContext<'input> for AccountDeclarationContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_accountDeclaration }
	//fn type_rule_index() -> usize where Self: Sized { RULE_accountDeclaration }
}
antlr4rust::tid!{AccountDeclarationContextExt<'a>}

impl<'input> AccountDeclarationContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<AccountDeclarationContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AccountDeclarationContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AccountDeclarationContextAttrs<'input>: LedgerContext<'input> + BorrowMut<AccountDeclarationContextExt<'input>>{

fn accountHeader(&self) -> Option<Rc<AccountHeaderContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn accountDetailLine_all(&self) ->  Vec<Rc<AccountDetailLineContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn accountDetailLine(&self, i: usize) -> Option<Rc<AccountDetailLineContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn blankLine_all(&self) ->  Vec<Rc<BlankLineContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn blankLine(&self, i: usize) -> Option<Rc<BlankLineContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> AccountDeclarationContextAttrs<'input> for AccountDeclarationContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn accountDeclaration(&mut self,)
	-> Result<Rc<AccountDeclarationContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AccountDeclarationContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 16, RULE_accountDeclaration);
        let mut _localctx: Rc<AccountDeclarationContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule accountHeader*/
			recog.base.set_state(157);
			recog.accountHeader()?;

			recog.base.set_state(161);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(11,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					/*InvokeRule accountDetailLine*/
					recog.base.set_state(158);
					recog.accountDetailLine()?;

					}
					} 
				}
				recog.base.set_state(163);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(11,&mut recog.base)?;
			}
			recog.base.set_state(167);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(12,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					/*InvokeRule blankLine*/
					recog.base.set_state(164);
					recog.blankLine()?;

					}
					} 
				}
				recog.base.set_state(169);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(12,&mut recog.base)?;
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- accountHeader ----------------
pub type AccountHeaderContextAll<'input> = AccountHeaderContext<'input>;


pub type AccountHeaderContext<'input> = BaseParserRuleContext<'input,AccountHeaderContextExt<'input>>;

#[derive(Clone)]
pub struct AccountHeaderContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for AccountHeaderContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for AccountHeaderContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for AccountHeaderContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_accountHeader(self);
	}
}

impl<'input> CustomRuleContext<'input> for AccountHeaderContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_accountHeader }
	//fn type_rule_index() -> usize where Self: Sized { RULE_accountHeader }
}
antlr4rust::tid!{AccountHeaderContextExt<'a>}

impl<'input> AccountHeaderContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<AccountHeaderContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AccountHeaderContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AccountHeaderContextAttrs<'input>: LedgerContext<'input> + BorrowMut<AccountHeaderContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token ACCOUNT
/// Returns `None` if there is no child corresponding to token ACCOUNT
fn ACCOUNT(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_ACCOUNT, 0)
}
/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, i)
}
fn account(&self) -> Option<Rc<AccountContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn commodity(&self) -> Option<Rc<CommodityContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn metaComment(&self) -> Option<Rc<MetaCommentContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> AccountHeaderContextAttrs<'input> for AccountHeaderContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn accountHeader(&mut self,)
	-> Result<Rc<AccountHeaderContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AccountHeaderContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 18, RULE_accountHeader);
        let mut _localctx: Rc<AccountHeaderContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(170);
			recog.base.match_token(Ledger_ACCOUNT,&mut recog.err_handler)?;

			recog.base.set_state(171);
			recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

			/*InvokeRule account*/
			recog.base.set_state(172);
			recog.account()?;

			recog.base.set_state(175);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(13,&mut recog.base)? {
				x if x == 1=>{
					{
					recog.base.set_state(173);
					recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

					/*InvokeRule commodity*/
					recog.base.set_state(174);
					recog.commodity()?;

					}
				}

				_ => {}
			}
			recog.base.set_state(180);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Ledger_WS {
				{
				{
				recog.base.set_state(177);
				recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(182);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(184);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Ledger_SEMI {
				{
				/*InvokeRule metaComment*/
				recog.base.set_state(183);
				recog.metaComment()?;

				}
			}

			/*InvokeRule lineEnd*/
			recog.base.set_state(186);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- accountDetailLine ----------------
pub type AccountDetailLineContextAll<'input> = AccountDetailLineContext<'input>;


pub type AccountDetailLineContext<'input> = BaseParserRuleContext<'input,AccountDetailLineContextExt<'input>>;

#[derive(Clone)]
pub struct AccountDetailLineContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for AccountDetailLineContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for AccountDetailLineContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for AccountDetailLineContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_accountDetailLine(self);
	}
}

impl<'input> CustomRuleContext<'input> for AccountDetailLineContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_accountDetailLine }
	//fn type_rule_index() -> usize where Self: Sized { RULE_accountDetailLine }
}
antlr4rust::tid!{AccountDetailLineContextExt<'a>}

impl<'input> AccountDetailLineContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<AccountDetailLineContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AccountDetailLineContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AccountDetailLineContextAttrs<'input>: LedgerContext<'input> + BorrowMut<AccountDetailLineContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, i)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn openingDetail(&self) -> Option<Rc<OpeningDetailContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn nameDetail(&self) -> Option<Rc<NameDetailContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn metaComment(&self) -> Option<Rc<MetaCommentContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> AccountDetailLineContextAttrs<'input> for AccountDetailLineContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn accountDetailLine(&mut self,)
	-> Result<Rc<AccountDetailLineContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AccountDetailLineContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 20, RULE_accountDetailLine);
        let mut _localctx: Rc<AccountDetailLineContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(188);
			recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

			recog.base.set_state(191);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Ledger_OPENING 
				=> {
					{
					/*InvokeRule openingDetail*/
					recog.base.set_state(189);
					recog.openingDetail()?;

					}
				}

			Ledger_NAME 
				=> {
					{
					/*InvokeRule nameDetail*/
					recog.base.set_state(190);
					recog.nameDetail()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			recog.base.set_state(196);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Ledger_WS {
				{
				{
				recog.base.set_state(193);
				recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(198);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(200);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Ledger_SEMI {
				{
				/*InvokeRule metaComment*/
				recog.base.set_state(199);
				recog.metaComment()?;

				}
			}

			/*InvokeRule lineEnd*/
			recog.base.set_state(202);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- openingDetail ----------------
pub type OpeningDetailContextAll<'input> = OpeningDetailContext<'input>;


pub type OpeningDetailContext<'input> = BaseParserRuleContext<'input,OpeningDetailContextExt<'input>>;

#[derive(Clone)]
pub struct OpeningDetailContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for OpeningDetailContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for OpeningDetailContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for OpeningDetailContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_openingDetail(self);
	}
}

impl<'input> CustomRuleContext<'input> for OpeningDetailContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_openingDetail }
	//fn type_rule_index() -> usize where Self: Sized { RULE_openingDetail }
}
antlr4rust::tid!{OpeningDetailContextExt<'a>}

impl<'input> OpeningDetailContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<OpeningDetailContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,OpeningDetailContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait OpeningDetailContextAttrs<'input>: LedgerContext<'input> + BorrowMut<OpeningDetailContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token OPENING
/// Returns `None` if there is no child corresponding to token OPENING
fn OPENING(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_OPENING, 0)
}
/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, i)
}
fn amount(&self) -> Option<Rc<AmountContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn commodity(&self) -> Option<Rc<CommodityContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> OpeningDetailContextAttrs<'input> for OpeningDetailContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn openingDetail(&mut self,)
	-> Result<Rc<OpeningDetailContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = OpeningDetailContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 22, RULE_openingDetail);
        let mut _localctx: Rc<OpeningDetailContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(204);
			recog.base.match_token(Ledger_OPENING,&mut recog.err_handler)?;

			recog.base.set_state(205);
			recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

			/*InvokeRule amount*/
			recog.base.set_state(206);
			recog.amount()?;

			recog.base.set_state(209);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(19,&mut recog.base)? {
				x if x == 1=>{
					{
					recog.base.set_state(207);
					recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

					/*InvokeRule commodity*/
					recog.base.set_state(208);
					recog.commodity()?;

					}
				}

				_ => {}
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- nameDetail ----------------
pub type NameDetailContextAll<'input> = NameDetailContext<'input>;


pub type NameDetailContext<'input> = BaseParserRuleContext<'input,NameDetailContextExt<'input>>;

#[derive(Clone)]
pub struct NameDetailContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for NameDetailContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for NameDetailContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for NameDetailContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_nameDetail(self);
	}
}

impl<'input> CustomRuleContext<'input> for NameDetailContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_nameDetail }
	//fn type_rule_index() -> usize where Self: Sized { RULE_nameDetail }
}
antlr4rust::tid!{NameDetailContextExt<'a>}

impl<'input> NameDetailContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<NameDetailContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,NameDetailContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait NameDetailContextAttrs<'input>: LedgerContext<'input> + BorrowMut<NameDetailContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token NAME
/// Returns `None` if there is no child corresponding to token NAME
fn NAME(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_NAME, 0)
}
/// Retrieves first TerminalNode corresponding to token WS
/// Returns `None` if there is no child corresponding to token WS
fn WS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, 0)
}
fn nameValue(&self) -> Option<Rc<NameValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> NameDetailContextAttrs<'input> for NameDetailContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn nameDetail(&mut self,)
	-> Result<Rc<NameDetailContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = NameDetailContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 24, RULE_nameDetail);
        let mut _localctx: Rc<NameDetailContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(211);
			recog.base.match_token(Ledger_NAME,&mut recog.err_handler)?;

			recog.base.set_state(212);
			recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

			/*InvokeRule nameValue*/
			recog.base.set_state(213);
			recog.nameValue()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- nameValue ----------------
pub type NameValueContextAll<'input> = NameValueContext<'input>;


pub type NameValueContext<'input> = BaseParserRuleContext<'input,NameValueContextExt<'input>>;

#[derive(Clone)]
pub struct NameValueContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for NameValueContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for NameValueContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for NameValueContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_nameValue(self);
	}
}

impl<'input> CustomRuleContext<'input> for NameValueContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_nameValue }
	//fn type_rule_index() -> usize where Self: Sized { RULE_nameValue }
}
antlr4rust::tid!{NameValueContextExt<'a>}

impl<'input> NameValueContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<NameValueContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,NameValueContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait NameValueContextAttrs<'input>: LedgerContext<'input> + BorrowMut<NameValueContextExt<'input>>{

fn headerField_all(&self) ->  Vec<Rc<HeaderFieldContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn headerField(&self, i: usize) -> Option<Rc<HeaderFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, i)
}

}

impl<'input> NameValueContextAttrs<'input> for NameValueContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn nameValue(&mut self,)
	-> Result<Rc<NameValueContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = NameValueContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 26, RULE_nameValue);
        let mut _localctx: Rc<NameValueContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule headerField*/
			recog.base.set_state(215);
			recog.headerField()?;

			recog.base.set_state(220);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(20,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(216);
					recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

					/*InvokeRule headerField*/
					recog.base.set_state(217);
					recog.headerField()?;

					}
					} 
				}
				recog.base.set_state(222);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(20,&mut recog.base)?;
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- directive ----------------
pub type DirectiveContextAll<'input> = DirectiveContext<'input>;


pub type DirectiveContext<'input> = BaseParserRuleContext<'input,DirectiveContextExt<'input>>;

#[derive(Clone)]
pub struct DirectiveContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for DirectiveContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for DirectiveContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for DirectiveContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_directive(self);
	}
}

impl<'input> CustomRuleContext<'input> for DirectiveContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_directive }
	//fn type_rule_index() -> usize where Self: Sized { RULE_directive }
}
antlr4rust::tid!{DirectiveContextExt<'a>}

impl<'input> DirectiveContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<DirectiveContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,DirectiveContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait DirectiveContextAttrs<'input>: LedgerContext<'input> + BorrowMut<DirectiveContextExt<'input>>{

fn metaComment(&self) -> Option<Rc<MetaCommentContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token WS
/// Returns `None` if there is no child corresponding to token WS
fn WS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, 0)
}

}

impl<'input> DirectiveContextAttrs<'input> for DirectiveContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn directive(&mut self,)
	-> Result<Rc<DirectiveContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = DirectiveContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 28, RULE_directive);
        let mut _localctx: Rc<DirectiveContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(224);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Ledger_WS {
				{
				recog.base.set_state(223);
				recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

				}
			}

			/*InvokeRule metaComment*/
			recog.base.set_state(226);
			recog.metaComment()?;

			/*InvokeRule lineEnd*/
			recog.base.set_state(227);
			recog.lineEnd()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- blankLine ----------------
pub type BlankLineContextAll<'input> = BlankLineContext<'input>;


pub type BlankLineContext<'input> = BaseParserRuleContext<'input,BlankLineContextExt<'input>>;

#[derive(Clone)]
pub struct BlankLineContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for BlankLineContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for BlankLineContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for BlankLineContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_blankLine(self);
	}
}

impl<'input> CustomRuleContext<'input> for BlankLineContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_blankLine }
	//fn type_rule_index() -> usize where Self: Sized { RULE_blankLine }
}
antlr4rust::tid!{BlankLineContextExt<'a>}

impl<'input> BlankLineContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<BlankLineContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,BlankLineContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait BlankLineContextAttrs<'input>: LedgerContext<'input> + BorrowMut<BlankLineContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token NEWLINE
/// Returns `None` if there is no child corresponding to token NEWLINE
fn NEWLINE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_NEWLINE, 0)
}
/// Retrieves first TerminalNode corresponding to token WS
/// Returns `None` if there is no child corresponding to token WS
fn WS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, 0)
}

}

impl<'input> BlankLineContextAttrs<'input> for BlankLineContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn blankLine(&mut self,)
	-> Result<Rc<BlankLineContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = BlankLineContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 30, RULE_blankLine);
        let mut _localctx: Rc<BlankLineContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(230);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Ledger_WS {
				{
				recog.base.set_state(229);
				recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

				}
			}

			recog.base.set_state(232);
			recog.base.match_token(Ledger_NEWLINE,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- metaComment ----------------
pub type MetaCommentContextAll<'input> = MetaCommentContext<'input>;


pub type MetaCommentContext<'input> = BaseParserRuleContext<'input,MetaCommentContextExt<'input>>;

#[derive(Clone)]
pub struct MetaCommentContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for MetaCommentContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for MetaCommentContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for MetaCommentContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_metaComment(self);
	}
}

impl<'input> CustomRuleContext<'input> for MetaCommentContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_metaComment }
	//fn type_rule_index() -> usize where Self: Sized { RULE_metaComment }
}
antlr4rust::tid!{MetaCommentContextExt<'a>}

impl<'input> MetaCommentContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<MetaCommentContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,MetaCommentContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait MetaCommentContextAttrs<'input>: LedgerContext<'input> + BorrowMut<MetaCommentContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token SEMI
/// Returns `None` if there is no child corresponding to token SEMI
fn SEMI(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_SEMI, 0)
}
/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, i)
}
fn anyToken_all(&self) ->  Vec<Rc<AnyTokenContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn anyToken(&self, i: usize) -> Option<Rc<AnyTokenContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> MetaCommentContextAttrs<'input> for MetaCommentContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn metaComment(&mut self,)
	-> Result<Rc<MetaCommentContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = MetaCommentContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 32, RULE_metaComment);
        let mut _localctx: Rc<MetaCommentContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(234);
			recog.base.match_token(Ledger_SEMI,&mut recog.err_handler)?;

			recog.base.set_state(239);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(24,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					recog.base.set_state(237);
					recog.err_handler.sync(&mut recog.base)?;
					match recog.base.input.la(1) {
					Ledger_WS 
						=> {
							{
							recog.base.set_state(235);
							recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

							}
						}

					Ledger_ACCOUNT |Ledger_OPENING |Ledger_NAME |Ledger_PRICE_DIRECTIVE |
					Ledger_DOUBLE_LBRACE |Ledger_DOUBLE_RBRACE |Ledger_ATAT |Ledger_LBRACE |
					Ledger_RBRACE |Ledger_AT |Ledger_SEMI |Ledger_COMMA |Ledger_COLON |
					Ledger_STAR |Ledger_BANG |Ledger_PLUS |Ledger_MINUS |Ledger_DOT |Ledger_DATETIME |
					Ledger_GROUPED_NUMBER |Ledger_NUMBER |Ledger_QUOTED |Ledger_ALNUM |
					Ledger_OTHER 
						=> {
							{
							/*InvokeRule anyToken*/
							recog.base.set_state(236);
							recog.anyToken()?;

							}
						}

						_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
					}
					} 
				}
				recog.base.set_state(241);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(24,&mut recog.base)?;
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- accountInput ----------------
pub type AccountInputContextAll<'input> = AccountInputContext<'input>;


pub type AccountInputContext<'input> = BaseParserRuleContext<'input,AccountInputContextExt<'input>>;

#[derive(Clone)]
pub struct AccountInputContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for AccountInputContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for AccountInputContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for AccountInputContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_accountInput(self);
	}
}

impl<'input> CustomRuleContext<'input> for AccountInputContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_accountInput }
	//fn type_rule_index() -> usize where Self: Sized { RULE_accountInput }
}
antlr4rust::tid!{AccountInputContextExt<'a>}

impl<'input> AccountInputContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<AccountInputContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AccountInputContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AccountInputContextAttrs<'input>: LedgerContext<'input> + BorrowMut<AccountInputContextExt<'input>>{

fn account(&self) -> Option<Rc<AccountContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_EOF, 0)
}

}

impl<'input> AccountInputContextAttrs<'input> for AccountInputContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn accountInput(&mut self,)
	-> Result<Rc<AccountInputContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AccountInputContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 34, RULE_accountInput);
        let mut _localctx: Rc<AccountInputContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule account*/
			recog.base.set_state(242);
			recog.account()?;

			recog.base.set_state(243);
			recog.base.match_token(Ledger_EOF,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- commodityInput ----------------
pub type CommodityInputContextAll<'input> = CommodityInputContext<'input>;


pub type CommodityInputContext<'input> = BaseParserRuleContext<'input,CommodityInputContextExt<'input>>;

#[derive(Clone)]
pub struct CommodityInputContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for CommodityInputContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for CommodityInputContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for CommodityInputContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_commodityInput(self);
	}
}

impl<'input> CustomRuleContext<'input> for CommodityInputContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_commodityInput }
	//fn type_rule_index() -> usize where Self: Sized { RULE_commodityInput }
}
antlr4rust::tid!{CommodityInputContextExt<'a>}

impl<'input> CommodityInputContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<CommodityInputContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CommodityInputContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CommodityInputContextAttrs<'input>: LedgerContext<'input> + BorrowMut<CommodityInputContextExt<'input>>{

fn commodity(&self) -> Option<Rc<CommodityContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_EOF, 0)
}

}

impl<'input> CommodityInputContextAttrs<'input> for CommodityInputContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn commodityInput(&mut self,)
	-> Result<Rc<CommodityInputContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CommodityInputContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 36, RULE_commodityInput);
        let mut _localctx: Rc<CommodityInputContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule commodity*/
			recog.base.set_state(245);
			recog.commodity()?;

			recog.base.set_state(246);
			recog.base.match_token(Ledger_EOF,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- amountInput ----------------
pub type AmountInputContextAll<'input> = AmountInputContext<'input>;


pub type AmountInputContext<'input> = BaseParserRuleContext<'input,AmountInputContextExt<'input>>;

#[derive(Clone)]
pub struct AmountInputContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for AmountInputContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for AmountInputContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for AmountInputContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_amountInput(self);
	}
}

impl<'input> CustomRuleContext<'input> for AmountInputContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_amountInput }
	//fn type_rule_index() -> usize where Self: Sized { RULE_amountInput }
}
antlr4rust::tid!{AmountInputContextExt<'a>}

impl<'input> AmountInputContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<AmountInputContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AmountInputContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AmountInputContextAttrs<'input>: LedgerContext<'input> + BorrowMut<AmountInputContextExt<'input>>{

fn amount(&self) -> Option<Rc<AmountContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_EOF, 0)
}

}

impl<'input> AmountInputContextAttrs<'input> for AmountInputContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn amountInput(&mut self,)
	-> Result<Rc<AmountInputContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AmountInputContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 38, RULE_amountInput);
        let mut _localctx: Rc<AmountInputContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule amount*/
			recog.base.set_state(248);
			recog.amount()?;

			recog.base.set_state(249);
			recog.base.match_token(Ledger_EOF,&mut recog.err_handler)?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- account ----------------
pub type AccountContextAll<'input> = AccountContext<'input>;


pub type AccountContext<'input> = BaseParserRuleContext<'input,AccountContextExt<'input>>;

#[derive(Clone)]
pub struct AccountContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for AccountContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for AccountContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for AccountContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_account(self);
	}
}

impl<'input> CustomRuleContext<'input> for AccountContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_account }
	//fn type_rule_index() -> usize where Self: Sized { RULE_account }
}
antlr4rust::tid!{AccountContextExt<'a>}

impl<'input> AccountContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<AccountContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AccountContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AccountContextAttrs<'input>: LedgerContext<'input> + BorrowMut<AccountContextExt<'input>>{

fn accountSegment_all(&self) ->  Vec<Rc<AccountSegmentContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn accountSegment(&self, i: usize) -> Option<Rc<AccountSegmentContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token COLON in current rule
fn COLON_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token COLON, starting from 0.
/// Returns `None` if number of children corresponding to token COLON is less or equal than `i`.
fn COLON(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_COLON, i)
}

}

impl<'input> AccountContextAttrs<'input> for AccountContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn account(&mut self,)
	-> Result<Rc<AccountContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AccountContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 40, RULE_account);
        let mut _localctx: Rc<AccountContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule accountSegment*/
			recog.base.set_state(251);
			recog.accountSegment()?;

			recog.base.set_state(252);
			recog.base.match_token(Ledger_COLON,&mut recog.err_handler)?;

			/*InvokeRule accountSegment*/
			recog.base.set_state(253);
			recog.accountSegment()?;

			recog.base.set_state(258);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Ledger_COLON {
				{
				{
				recog.base.set_state(254);
				recog.base.match_token(Ledger_COLON,&mut recog.err_handler)?;

				/*InvokeRule accountSegment*/
				recog.base.set_state(255);
				recog.accountSegment()?;

				}
				}
				recog.base.set_state(260);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- accountSegment ----------------
pub type AccountSegmentContextAll<'input> = AccountSegmentContext<'input>;


pub type AccountSegmentContext<'input> = BaseParserRuleContext<'input,AccountSegmentContextExt<'input>>;

#[derive(Clone)]
pub struct AccountSegmentContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for AccountSegmentContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for AccountSegmentContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for AccountSegmentContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_accountSegment(self);
	}
}

impl<'input> CustomRuleContext<'input> for AccountSegmentContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_accountSegment }
	//fn type_rule_index() -> usize where Self: Sized { RULE_accountSegment }
}
antlr4rust::tid!{AccountSegmentContextExt<'a>}

impl<'input> AccountSegmentContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<AccountSegmentContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AccountSegmentContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AccountSegmentContextAttrs<'input>: LedgerContext<'input> + BorrowMut<AccountSegmentContextExt<'input>>{

fn accountPiece_all(&self) ->  Vec<Rc<AccountPieceContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn accountPiece(&self, i: usize) -> Option<Rc<AccountPieceContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> AccountSegmentContextAttrs<'input> for AccountSegmentContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn accountSegment(&mut self,)
	-> Result<Rc<AccountSegmentContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AccountSegmentContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 42, RULE_accountSegment);
        let mut _localctx: Rc<AccountSegmentContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(262); 
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			loop {
				{
				{
				/*InvokeRule accountPiece*/
				recog.base.set_state(261);
				recog.accountPiece()?;

				}
				}
				recog.base.set_state(264); 
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				if !((((_la) & !0x3f) == 0 && ((1usize << _la) & 10879006) != 0)) {break}
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- accountPiece ----------------
pub type AccountPieceContextAll<'input> = AccountPieceContext<'input>;


pub type AccountPieceContext<'input> = BaseParserRuleContext<'input,AccountPieceContextExt<'input>>;

#[derive(Clone)]
pub struct AccountPieceContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for AccountPieceContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for AccountPieceContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for AccountPieceContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_accountPiece(self);
	}
}

impl<'input> CustomRuleContext<'input> for AccountPieceContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_accountPiece }
	//fn type_rule_index() -> usize where Self: Sized { RULE_accountPiece }
}
antlr4rust::tid!{AccountPieceContextExt<'a>}

impl<'input> AccountPieceContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<AccountPieceContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AccountPieceContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AccountPieceContextAttrs<'input>: LedgerContext<'input> + BorrowMut<AccountPieceContextExt<'input>>{

fn wordToken(&self) -> Option<Rc<WordTokenContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token NUMBER
/// Returns `None` if there is no child corresponding to token NUMBER
fn NUMBER(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token DOT
/// Returns `None` if there is no child corresponding to token DOT
fn DOT(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DOT, 0)
}
/// Retrieves first TerminalNode corresponding to token MINUS
/// Returns `None` if there is no child corresponding to token MINUS
fn MINUS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_MINUS, 0)
}

}

impl<'input> AccountPieceContextAttrs<'input> for AccountPieceContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn accountPiece(&mut self,)
	-> Result<Rc<AccountPieceContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AccountPieceContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 44, RULE_accountPiece);
        let mut _localctx: Rc<AccountPieceContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(270);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Ledger_ACCOUNT |Ledger_OPENING |Ledger_NAME |Ledger_PRICE_DIRECTIVE |
			Ledger_ALNUM 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule wordToken*/
					recog.base.set_state(266);
					recog.wordToken()?;

					}
				}

			Ledger_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(267);
					recog.base.match_token(Ledger_NUMBER,&mut recog.err_handler)?;

					}
				}

			Ledger_DOT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(268);
					recog.base.match_token(Ledger_DOT,&mut recog.err_handler)?;

					}
				}

			Ledger_MINUS 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(269);
					recog.base.match_token(Ledger_MINUS,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- commodity ----------------
pub type CommodityContextAll<'input> = CommodityContext<'input>;


pub type CommodityContext<'input> = BaseParserRuleContext<'input,CommodityContextExt<'input>>;

#[derive(Clone)]
pub struct CommodityContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for CommodityContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for CommodityContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for CommodityContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_commodity(self);
	}
}

impl<'input> CustomRuleContext<'input> for CommodityContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_commodity }
	//fn type_rule_index() -> usize where Self: Sized { RULE_commodity }
}
antlr4rust::tid!{CommodityContextExt<'a>}

impl<'input> CommodityContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<CommodityContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CommodityContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CommodityContextAttrs<'input>: LedgerContext<'input> + BorrowMut<CommodityContextExt<'input>>{

fn commodityHead(&self) -> Option<Rc<CommodityHeadContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn commodityTail_all(&self) ->  Vec<Rc<CommodityTailContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn commodityTail(&self, i: usize) -> Option<Rc<CommodityTailContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> CommodityContextAttrs<'input> for CommodityContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn commodity(&mut self,)
	-> Result<Rc<CommodityContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CommodityContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 46, RULE_commodity);
        let mut _localctx: Rc<CommodityContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule commodityHead*/
			recog.base.set_state(272);
			recog.commodityHead()?;

			recog.base.set_state(276);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(28,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					/*InvokeRule commodityTail*/
					recog.base.set_state(273);
					recog.commodityTail()?;

					}
					} 
				}
				recog.base.set_state(278);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(28,&mut recog.base)?;
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- commodityHead ----------------
pub type CommodityHeadContextAll<'input> = CommodityHeadContext<'input>;


pub type CommodityHeadContext<'input> = BaseParserRuleContext<'input,CommodityHeadContextExt<'input>>;

#[derive(Clone)]
pub struct CommodityHeadContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for CommodityHeadContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for CommodityHeadContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for CommodityHeadContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_commodityHead(self);
	}
}

impl<'input> CustomRuleContext<'input> for CommodityHeadContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_commodityHead }
	//fn type_rule_index() -> usize where Self: Sized { RULE_commodityHead }
}
antlr4rust::tid!{CommodityHeadContextExt<'a>}

impl<'input> CommodityHeadContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<CommodityHeadContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CommodityHeadContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CommodityHeadContextAttrs<'input>: LedgerContext<'input> + BorrowMut<CommodityHeadContextExt<'input>>{

fn wordToken(&self) -> Option<Rc<WordTokenContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token NUMBER
/// Returns `None` if there is no child corresponding to token NUMBER
fn NUMBER(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_NUMBER, 0)
}

}

impl<'input> CommodityHeadContextAttrs<'input> for CommodityHeadContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn commodityHead(&mut self,)
	-> Result<Rc<CommodityHeadContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CommodityHeadContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 48, RULE_commodityHead);
        let mut _localctx: Rc<CommodityHeadContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(281);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Ledger_ACCOUNT |Ledger_OPENING |Ledger_NAME |Ledger_PRICE_DIRECTIVE |
			Ledger_ALNUM 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule wordToken*/
					recog.base.set_state(279);
					recog.wordToken()?;

					}
				}

			Ledger_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(280);
					recog.base.match_token(Ledger_NUMBER,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- commodityTail ----------------
pub type CommodityTailContextAll<'input> = CommodityTailContext<'input>;


pub type CommodityTailContext<'input> = BaseParserRuleContext<'input,CommodityTailContextExt<'input>>;

#[derive(Clone)]
pub struct CommodityTailContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for CommodityTailContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for CommodityTailContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for CommodityTailContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_commodityTail(self);
	}
}

impl<'input> CustomRuleContext<'input> for CommodityTailContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_commodityTail }
	//fn type_rule_index() -> usize where Self: Sized { RULE_commodityTail }
}
antlr4rust::tid!{CommodityTailContextExt<'a>}

impl<'input> CommodityTailContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<CommodityTailContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CommodityTailContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CommodityTailContextAttrs<'input>: LedgerContext<'input> + BorrowMut<CommodityTailContextExt<'input>>{

fn commodityHead(&self) -> Option<Rc<CommodityHeadContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token DOT
/// Returns `None` if there is no child corresponding to token DOT
fn DOT(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DOT, 0)
}
/// Retrieves first TerminalNode corresponding to token MINUS
/// Returns `None` if there is no child corresponding to token MINUS
fn MINUS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_MINUS, 0)
}
/// Retrieves first TerminalNode corresponding to token COLON
/// Returns `None` if there is no child corresponding to token COLON
fn COLON(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_COLON, 0)
}

}

impl<'input> CommodityTailContextAttrs<'input> for CommodityTailContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn commodityTail(&mut self,)
	-> Result<Rc<CommodityTailContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CommodityTailContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 50, RULE_commodityTail);
        let mut _localctx: Rc<CommodityTailContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(287);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Ledger_ACCOUNT |Ledger_OPENING |Ledger_NAME |Ledger_PRICE_DIRECTIVE |
			Ledger_NUMBER |Ledger_ALNUM 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule commodityHead*/
					recog.base.set_state(283);
					recog.commodityHead()?;

					}
				}

			Ledger_DOT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(284);
					recog.base.match_token(Ledger_DOT,&mut recog.err_handler)?;

					}
				}

			Ledger_MINUS 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(285);
					recog.base.match_token(Ledger_MINUS,&mut recog.err_handler)?;

					}
				}

			Ledger_COLON 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(286);
					recog.base.match_token(Ledger_COLON,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- amount ----------------
pub type AmountContextAll<'input> = AmountContext<'input>;


pub type AmountContext<'input> = BaseParserRuleContext<'input,AmountContextExt<'input>>;

#[derive(Clone)]
pub struct AmountContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for AmountContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for AmountContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for AmountContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_amount(self);
	}
}

impl<'input> CustomRuleContext<'input> for AmountContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_amount }
	//fn type_rule_index() -> usize where Self: Sized { RULE_amount }
}
antlr4rust::tid!{AmountContextExt<'a>}

impl<'input> AmountContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<AmountContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AmountContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AmountContextAttrs<'input>: LedgerContext<'input> + BorrowMut<AmountContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token NUMBER
/// Returns `None` if there is no child corresponding to token NUMBER
fn NUMBER(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token GROUPED_NUMBER
/// Returns `None` if there is no child corresponding to token GROUPED_NUMBER
fn GROUPED_NUMBER(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_GROUPED_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token PLUS
/// Returns `None` if there is no child corresponding to token PLUS
fn PLUS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_PLUS, 0)
}
/// Retrieves first TerminalNode corresponding to token MINUS
/// Returns `None` if there is no child corresponding to token MINUS
fn MINUS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_MINUS, 0)
}

}

impl<'input> AmountContextAttrs<'input> for AmountContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn amount(&mut self,)
	-> Result<Rc<AmountContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AmountContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 52, RULE_amount);
        let mut _localctx: Rc<AmountContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(290);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Ledger_PLUS || _la==Ledger_MINUS {
				{
				recog.base.set_state(289);
				_la = recog.base.input.la(1);
				if { !(_la==Ledger_PLUS || _la==Ledger_MINUS) } {
					recog.err_handler.recover_inline(&mut recog.base)?;

				}
				else {
					if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
					recog.err_handler.report_match(&mut recog.base);
					recog.base.consume(&mut recog.err_handler);
				}
				}
			}

			recog.base.set_state(292);
			_la = recog.base.input.la(1);
			if { !(_la==Ledger_GROUPED_NUMBER || _la==Ledger_NUMBER) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- lotCostSpec ----------------
pub type LotCostSpecContextAll<'input> = LotCostSpecContext<'input>;


pub type LotCostSpecContext<'input> = BaseParserRuleContext<'input,LotCostSpecContextExt<'input>>;

#[derive(Clone)]
pub struct LotCostSpecContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for LotCostSpecContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for LotCostSpecContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for LotCostSpecContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_lotCostSpec(self);
	}
}

impl<'input> CustomRuleContext<'input> for LotCostSpecContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_lotCostSpec }
	//fn type_rule_index() -> usize where Self: Sized { RULE_lotCostSpec }
}
antlr4rust::tid!{LotCostSpecContextExt<'a>}

impl<'input> LotCostSpecContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<LotCostSpecContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,LotCostSpecContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait LotCostSpecContextAttrs<'input>: LedgerContext<'input> + BorrowMut<LotCostSpecContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token LBRACE
/// Returns `None` if there is no child corresponding to token LBRACE
fn LBRACE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_LBRACE, 0)
}
fn costBody(&self) -> Option<Rc<CostBodyContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token RBRACE
/// Returns `None` if there is no child corresponding to token RBRACE
fn RBRACE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_RBRACE, 0)
}
/// Retrieves first TerminalNode corresponding to token DOUBLE_LBRACE
/// Returns `None` if there is no child corresponding to token DOUBLE_LBRACE
fn DOUBLE_LBRACE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DOUBLE_LBRACE, 0)
}
/// Retrieves first TerminalNode corresponding to token DOUBLE_RBRACE
/// Returns `None` if there is no child corresponding to token DOUBLE_RBRACE
fn DOUBLE_RBRACE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DOUBLE_RBRACE, 0)
}

}

impl<'input> LotCostSpecContextAttrs<'input> for LotCostSpecContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn lotCostSpec(&mut self,)
	-> Result<Rc<LotCostSpecContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = LotCostSpecContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 54, RULE_lotCostSpec);
        let mut _localctx: Rc<LotCostSpecContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(302);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Ledger_LBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(294);
					recog.base.match_token(Ledger_LBRACE,&mut recog.err_handler)?;

					/*InvokeRule costBody*/
					recog.base.set_state(295);
					recog.costBody()?;

					recog.base.set_state(296);
					recog.base.match_token(Ledger_RBRACE,&mut recog.err_handler)?;

					}
				}

			Ledger_DOUBLE_LBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(298);
					recog.base.match_token(Ledger_DOUBLE_LBRACE,&mut recog.err_handler)?;

					/*InvokeRule costBody*/
					recog.base.set_state(299);
					recog.costBody()?;

					recog.base.set_state(300);
					recog.base.match_token(Ledger_DOUBLE_RBRACE,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- costBody ----------------
pub type CostBodyContextAll<'input> = CostBodyContext<'input>;


pub type CostBodyContext<'input> = BaseParserRuleContext<'input,CostBodyContextExt<'input>>;

#[derive(Clone)]
pub struct CostBodyContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for CostBodyContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for CostBodyContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for CostBodyContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_costBody(self);
	}
}

impl<'input> CustomRuleContext<'input> for CostBodyContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_costBody }
	//fn type_rule_index() -> usize where Self: Sized { RULE_costBody }
}
antlr4rust::tid!{CostBodyContextExt<'a>}

impl<'input> CostBodyContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<CostBodyContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CostBodyContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CostBodyContextAttrs<'input>: LedgerContext<'input> + BorrowMut<CostBodyContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, i)
}
fn costField_all(&self) ->  Vec<Rc<CostFieldContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn costField(&self, i: usize) -> Option<Rc<CostFieldContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token COMMA in current rule
fn COMMA_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token COMMA, starting from 0.
/// Returns `None` if number of children corresponding to token COMMA is less or equal than `i`.
fn COMMA(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_COMMA, i)
}

}

impl<'input> CostBodyContextAttrs<'input> for CostBodyContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn costBody(&mut self,)
	-> Result<Rc<CostBodyContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CostBodyContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 56, RULE_costBody);
        let mut _localctx: Rc<CostBodyContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(307);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(33,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(304);
					recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

					}
					} 
				}
				recog.base.set_state(309);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(33,&mut recog.base)?;
			}
			recog.base.set_state(311);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if (((_la) & !0x3f) == 0 && ((1usize << _la) & 83828766) != 0) {
				{
				/*InvokeRule costField*/
				recog.base.set_state(310);
				recog.costField()?;

				}
			}

			recog.base.set_state(329);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(37,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(316);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==Ledger_WS {
						{
						{
						recog.base.set_state(313);
						recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(318);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					recog.base.set_state(319);
					recog.base.match_token(Ledger_COMMA,&mut recog.err_handler)?;

					recog.base.set_state(323);
					recog.err_handler.sync(&mut recog.base)?;
					_la = recog.base.input.la(1);
					while _la==Ledger_WS {
						{
						{
						recog.base.set_state(320);
						recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

						}
						}
						recog.base.set_state(325);
						recog.err_handler.sync(&mut recog.base)?;
						_la = recog.base.input.la(1);
					}
					/*InvokeRule costField*/
					recog.base.set_state(326);
					recog.costField()?;

					}
					} 
				}
				recog.base.set_state(331);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(37,&mut recog.base)?;
			}
			recog.base.set_state(335);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Ledger_WS {
				{
				{
				recog.base.set_state(332);
				recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(337);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- costField ----------------
pub type CostFieldContextAll<'input> = CostFieldContext<'input>;


pub type CostFieldContext<'input> = BaseParserRuleContext<'input,CostFieldContextExt<'input>>;

#[derive(Clone)]
pub struct CostFieldContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for CostFieldContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for CostFieldContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for CostFieldContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_costField(self);
	}
}

impl<'input> CustomRuleContext<'input> for CostFieldContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_costField }
	//fn type_rule_index() -> usize where Self: Sized { RULE_costField }
}
antlr4rust::tid!{CostFieldContextExt<'a>}

impl<'input> CostFieldContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<CostFieldContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CostFieldContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CostFieldContextAttrs<'input>: LedgerContext<'input> + BorrowMut<CostFieldContextExt<'input>>{

fn costValue_all(&self) ->  Vec<Rc<CostValueContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn costValue(&self, i: usize) -> Option<Rc<CostValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, i)
}

}

impl<'input> CostFieldContextAttrs<'input> for CostFieldContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn costField(&mut self,)
	-> Result<Rc<CostFieldContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CostFieldContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 58, RULE_costField);
        let mut _localctx: Rc<CostFieldContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule costValue*/
			recog.base.set_state(338);
			recog.costValue()?;

			recog.base.set_state(343);
			recog.err_handler.sync(&mut recog.base)?;
			_alt = recog.interpreter.adaptive_predict(39,&mut recog.base)?;
			while { _alt!=2 && _alt!=INVALID_ALT } {
				if _alt==1 {
					{
					{
					recog.base.set_state(339);
					recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

					/*InvokeRule costValue*/
					recog.base.set_state(340);
					recog.costValue()?;

					}
					} 
				}
				recog.base.set_state(345);
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(39,&mut recog.base)?;
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- costValue ----------------
pub type CostValueContextAll<'input> = CostValueContext<'input>;


pub type CostValueContext<'input> = BaseParserRuleContext<'input,CostValueContextExt<'input>>;

#[derive(Clone)]
pub struct CostValueContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for CostValueContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for CostValueContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for CostValueContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_costValue(self);
	}
}

impl<'input> CustomRuleContext<'input> for CostValueContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_costValue }
	//fn type_rule_index() -> usize where Self: Sized { RULE_costValue }
}
antlr4rust::tid!{CostValueContextExt<'a>}

impl<'input> CostValueContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<CostValueContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CostValueContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CostValueContextAttrs<'input>: LedgerContext<'input> + BorrowMut<CostValueContextExt<'input>>{

/// Retrieves all `TerminalNode`s corresponding to token QUOTED in current rule
fn QUOTED_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token QUOTED, starting from 0.
/// Returns `None` if number of children corresponding to token QUOTED is less or equal than `i`.
fn QUOTED(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_QUOTED, i)
}
fn barePiece_all(&self) ->  Vec<Rc<BarePieceContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn barePiece(&self, i: usize) -> Option<Rc<BarePieceContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> CostValueContextAttrs<'input> for CostValueContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn costValue(&mut self,)
	-> Result<Rc<CostValueContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CostValueContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 60, RULE_costValue);
        let mut _localctx: Rc<CostValueContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(348); 
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			loop {
				{
				recog.base.set_state(348);
				recog.err_handler.sync(&mut recog.base)?;
				match recog.base.input.la(1) {
				Ledger_QUOTED 
					=> {
						{
						recog.base.set_state(346);
						recog.base.match_token(Ledger_QUOTED,&mut recog.err_handler)?;

						}
					}

				Ledger_ACCOUNT |Ledger_OPENING |Ledger_NAME |Ledger_PRICE_DIRECTIVE |
				Ledger_COLON |Ledger_PLUS |Ledger_MINUS |Ledger_DOT |Ledger_DATETIME |
				Ledger_GROUPED_NUMBER |Ledger_NUMBER |Ledger_ALNUM |Ledger_OTHER 
					=> {
						{
						/*InvokeRule barePiece*/
						recog.base.set_state(347);
						recog.barePiece()?;

						}
					}

					_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
				}
				}
				recog.base.set_state(350); 
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
				if !((((_la) & !0x3f) == 0 && ((1usize << _la) & 83828766) != 0)) {break}
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- priceSpec ----------------
pub type PriceSpecContextAll<'input> = PriceSpecContext<'input>;


pub type PriceSpecContext<'input> = BaseParserRuleContext<'input,PriceSpecContextExt<'input>>;

#[derive(Clone)]
pub struct PriceSpecContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for PriceSpecContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for PriceSpecContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for PriceSpecContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_priceSpec(self);
	}
}

impl<'input> CustomRuleContext<'input> for PriceSpecContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_priceSpec }
	//fn type_rule_index() -> usize where Self: Sized { RULE_priceSpec }
}
antlr4rust::tid!{PriceSpecContextExt<'a>}

impl<'input> PriceSpecContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<PriceSpecContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PriceSpecContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PriceSpecContextAttrs<'input>: LedgerContext<'input> + BorrowMut<PriceSpecContextExt<'input>>{

fn amount(&self) -> Option<Rc<AmountContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,LedgerContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_WS, i)
}
fn commodity(&self) -> Option<Rc<CommodityContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token AT
/// Returns `None` if there is no child corresponding to token AT
fn AT(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_AT, 0)
}
/// Retrieves first TerminalNode corresponding to token ATAT
/// Returns `None` if there is no child corresponding to token ATAT
fn ATAT(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_ATAT, 0)
}

}

impl<'input> PriceSpecContextAttrs<'input> for PriceSpecContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn priceSpec(&mut self,)
	-> Result<Rc<PriceSpecContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PriceSpecContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 62, RULE_priceSpec);
        let mut _localctx: Rc<PriceSpecContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(352);
			_la = recog.base.input.la(1);
			if { !(_la==Ledger_ATAT || _la==Ledger_AT) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			recog.base.set_state(356);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Ledger_WS {
				{
				{
				recog.base.set_state(353);
				recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(358);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule amount*/
			recog.base.set_state(359);
			recog.amount()?;

			recog.base.set_state(360);
			recog.base.match_token(Ledger_WS,&mut recog.err_handler)?;

			/*InvokeRule commodity*/
			recog.base.set_state(361);
			recog.commodity()?;

			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- bareValue ----------------
pub type BareValueContextAll<'input> = BareValueContext<'input>;


pub type BareValueContext<'input> = BaseParserRuleContext<'input,BareValueContextExt<'input>>;

#[derive(Clone)]
pub struct BareValueContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for BareValueContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for BareValueContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for BareValueContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_bareValue(self);
	}
}

impl<'input> CustomRuleContext<'input> for BareValueContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_bareValue }
	//fn type_rule_index() -> usize where Self: Sized { RULE_bareValue }
}
antlr4rust::tid!{BareValueContextExt<'a>}

impl<'input> BareValueContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<BareValueContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,BareValueContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait BareValueContextAttrs<'input>: LedgerContext<'input> + BorrowMut<BareValueContextExt<'input>>{

fn barePiece_all(&self) ->  Vec<Rc<BarePieceContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn barePiece(&self, i: usize) -> Option<Rc<BarePieceContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> BareValueContextAttrs<'input> for BareValueContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn bareValue(&mut self,)
	-> Result<Rc<BareValueContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = BareValueContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 64, RULE_bareValue);
        let mut _localctx: Rc<BareValueContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			let mut _alt: i32;
			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(364); 
			recog.err_handler.sync(&mut recog.base)?;
			_alt = 1;
			loop {
				match _alt {
				    x if x == 1=>
					{
					{
					/*InvokeRule barePiece*/
					recog.base.set_state(363);
					recog.barePiece()?;

					}
					}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
				}
				recog.base.set_state(366); 
				recog.err_handler.sync(&mut recog.base)?;
				_alt = recog.interpreter.adaptive_predict(43,&mut recog.base)?;
				if _alt==2 || _alt==INVALID_ALT { break }
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- barePiece ----------------
pub type BarePieceContextAll<'input> = BarePieceContext<'input>;


pub type BarePieceContext<'input> = BaseParserRuleContext<'input,BarePieceContextExt<'input>>;

#[derive(Clone)]
pub struct BarePieceContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for BarePieceContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for BarePieceContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for BarePieceContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_barePiece(self);
	}
}

impl<'input> CustomRuleContext<'input> for BarePieceContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_barePiece }
	//fn type_rule_index() -> usize where Self: Sized { RULE_barePiece }
}
antlr4rust::tid!{BarePieceContextExt<'a>}

impl<'input> BarePieceContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<BarePieceContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,BarePieceContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait BarePieceContextAttrs<'input>: LedgerContext<'input> + BorrowMut<BarePieceContextExt<'input>>{

fn wordToken(&self) -> Option<Rc<WordTokenContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token NUMBER
/// Returns `None` if there is no child corresponding to token NUMBER
fn NUMBER(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token GROUPED_NUMBER
/// Returns `None` if there is no child corresponding to token GROUPED_NUMBER
fn GROUPED_NUMBER(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_GROUPED_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token DATETIME
/// Returns `None` if there is no child corresponding to token DATETIME
fn DATETIME(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DATETIME, 0)
}
/// Retrieves first TerminalNode corresponding to token DOT
/// Returns `None` if there is no child corresponding to token DOT
fn DOT(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DOT, 0)
}
/// Retrieves first TerminalNode corresponding to token MINUS
/// Returns `None` if there is no child corresponding to token MINUS
fn MINUS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_MINUS, 0)
}
/// Retrieves first TerminalNode corresponding to token PLUS
/// Returns `None` if there is no child corresponding to token PLUS
fn PLUS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_PLUS, 0)
}
/// Retrieves first TerminalNode corresponding to token COLON
/// Returns `None` if there is no child corresponding to token COLON
fn COLON(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_COLON, 0)
}
/// Retrieves first TerminalNode corresponding to token OTHER
/// Returns `None` if there is no child corresponding to token OTHER
fn OTHER(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_OTHER, 0)
}

}

impl<'input> BarePieceContextAttrs<'input> for BarePieceContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn barePiece(&mut self,)
	-> Result<Rc<BarePieceContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = BarePieceContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 66, RULE_barePiece);
        let mut _localctx: Rc<BarePieceContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(377);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Ledger_ACCOUNT |Ledger_OPENING |Ledger_NAME |Ledger_PRICE_DIRECTIVE |
			Ledger_ALNUM 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule wordToken*/
					recog.base.set_state(368);
					recog.wordToken()?;

					}
				}

			Ledger_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(369);
					recog.base.match_token(Ledger_NUMBER,&mut recog.err_handler)?;

					}
				}

			Ledger_GROUPED_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(370);
					recog.base.match_token(Ledger_GROUPED_NUMBER,&mut recog.err_handler)?;

					}
				}

			Ledger_DATETIME 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(371);
					recog.base.match_token(Ledger_DATETIME,&mut recog.err_handler)?;

					}
				}

			Ledger_DOT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					recog.base.set_state(372);
					recog.base.match_token(Ledger_DOT,&mut recog.err_handler)?;

					}
				}

			Ledger_MINUS 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					recog.base.set_state(373);
					recog.base.match_token(Ledger_MINUS,&mut recog.err_handler)?;

					}
				}

			Ledger_PLUS 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					recog.base.set_state(374);
					recog.base.match_token(Ledger_PLUS,&mut recog.err_handler)?;

					}
				}

			Ledger_COLON 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					recog.base.set_state(375);
					recog.base.match_token(Ledger_COLON,&mut recog.err_handler)?;

					}
				}

			Ledger_OTHER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 9)?;
					recog.base.enter_outer_alt(None, 9)?;
					{
					recog.base.set_state(376);
					recog.base.match_token(Ledger_OTHER,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- wordToken ----------------
pub type WordTokenContextAll<'input> = WordTokenContext<'input>;


pub type WordTokenContext<'input> = BaseParserRuleContext<'input,WordTokenContextExt<'input>>;

#[derive(Clone)]
pub struct WordTokenContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for WordTokenContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for WordTokenContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for WordTokenContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_wordToken(self);
	}
}

impl<'input> CustomRuleContext<'input> for WordTokenContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_wordToken }
	//fn type_rule_index() -> usize where Self: Sized { RULE_wordToken }
}
antlr4rust::tid!{WordTokenContextExt<'a>}

impl<'input> WordTokenContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<WordTokenContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,WordTokenContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait WordTokenContextAttrs<'input>: LedgerContext<'input> + BorrowMut<WordTokenContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token ALNUM
/// Returns `None` if there is no child corresponding to token ALNUM
fn ALNUM(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_ALNUM, 0)
}
/// Retrieves first TerminalNode corresponding to token ACCOUNT
/// Returns `None` if there is no child corresponding to token ACCOUNT
fn ACCOUNT(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_ACCOUNT, 0)
}
/// Retrieves first TerminalNode corresponding to token OPENING
/// Returns `None` if there is no child corresponding to token OPENING
fn OPENING(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_OPENING, 0)
}
/// Retrieves first TerminalNode corresponding to token NAME
/// Returns `None` if there is no child corresponding to token NAME
fn NAME(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_NAME, 0)
}
/// Retrieves first TerminalNode corresponding to token PRICE_DIRECTIVE
/// Returns `None` if there is no child corresponding to token PRICE_DIRECTIVE
fn PRICE_DIRECTIVE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_PRICE_DIRECTIVE, 0)
}

}

impl<'input> WordTokenContextAttrs<'input> for WordTokenContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn wordToken(&mut self,)
	-> Result<Rc<WordTokenContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = WordTokenContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 68, RULE_wordToken);
        let mut _localctx: Rc<WordTokenContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(379);
			_la = recog.base.input.la(1);
			if { !((((_la) & !0x3f) == 0 && ((1usize << _la) & 8388638) != 0)) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- looseToken ----------------
pub type LooseTokenContextAll<'input> = LooseTokenContext<'input>;


pub type LooseTokenContext<'input> = BaseParserRuleContext<'input,LooseTokenContextExt<'input>>;

#[derive(Clone)]
pub struct LooseTokenContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for LooseTokenContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for LooseTokenContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for LooseTokenContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_looseToken(self);
	}
}

impl<'input> CustomRuleContext<'input> for LooseTokenContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_looseToken }
	//fn type_rule_index() -> usize where Self: Sized { RULE_looseToken }
}
antlr4rust::tid!{LooseTokenContextExt<'a>}

impl<'input> LooseTokenContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<LooseTokenContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,LooseTokenContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait LooseTokenContextAttrs<'input>: LedgerContext<'input> + BorrowMut<LooseTokenContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token QUOTED
/// Returns `None` if there is no child corresponding to token QUOTED
fn QUOTED(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_QUOTED, 0)
}
/// Retrieves first TerminalNode corresponding to token COMMA
/// Returns `None` if there is no child corresponding to token COMMA
fn COMMA(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_COMMA, 0)
}
fn bareValue(&self) -> Option<Rc<BareValueContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> LooseTokenContextAttrs<'input> for LooseTokenContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn looseToken(&mut self,)
	-> Result<Rc<LooseTokenContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = LooseTokenContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 70, RULE_looseToken);
        let mut _localctx: Rc<LooseTokenContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(384);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Ledger_QUOTED 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					recog.base.set_state(381);
					recog.base.match_token(Ledger_QUOTED,&mut recog.err_handler)?;

					}
				}

			Ledger_COMMA 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(382);
					recog.base.match_token(Ledger_COMMA,&mut recog.err_handler)?;

					}
				}

			Ledger_ACCOUNT |Ledger_OPENING |Ledger_NAME |Ledger_PRICE_DIRECTIVE |
			Ledger_COLON |Ledger_PLUS |Ledger_MINUS |Ledger_DOT |Ledger_DATETIME |
			Ledger_GROUPED_NUMBER |Ledger_NUMBER |Ledger_ALNUM |Ledger_OTHER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule bareValue*/
					recog.base.set_state(383);
					recog.bareValue()?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- anyToken ----------------
pub type AnyTokenContextAll<'input> = AnyTokenContext<'input>;


pub type AnyTokenContext<'input> = BaseParserRuleContext<'input,AnyTokenContextExt<'input>>;

#[derive(Clone)]
pub struct AnyTokenContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for AnyTokenContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for AnyTokenContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for AnyTokenContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_anyToken(self);
	}
}

impl<'input> CustomRuleContext<'input> for AnyTokenContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_anyToken }
	//fn type_rule_index() -> usize where Self: Sized { RULE_anyToken }
}
antlr4rust::tid!{AnyTokenContextExt<'a>}

impl<'input> AnyTokenContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<AnyTokenContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AnyTokenContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AnyTokenContextAttrs<'input>: LedgerContext<'input> + BorrowMut<AnyTokenContextExt<'input>>{

fn wordToken(&self) -> Option<Rc<WordTokenContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token NUMBER
/// Returns `None` if there is no child corresponding to token NUMBER
fn NUMBER(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token GROUPED_NUMBER
/// Returns `None` if there is no child corresponding to token GROUPED_NUMBER
fn GROUPED_NUMBER(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_GROUPED_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token DATETIME
/// Returns `None` if there is no child corresponding to token DATETIME
fn DATETIME(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DATETIME, 0)
}
/// Retrieves first TerminalNode corresponding to token QUOTED
/// Returns `None` if there is no child corresponding to token QUOTED
fn QUOTED(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_QUOTED, 0)
}
/// Retrieves first TerminalNode corresponding to token DOUBLE_LBRACE
/// Returns `None` if there is no child corresponding to token DOUBLE_LBRACE
fn DOUBLE_LBRACE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DOUBLE_LBRACE, 0)
}
/// Retrieves first TerminalNode corresponding to token DOUBLE_RBRACE
/// Returns `None` if there is no child corresponding to token DOUBLE_RBRACE
fn DOUBLE_RBRACE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DOUBLE_RBRACE, 0)
}
/// Retrieves first TerminalNode corresponding to token ATAT
/// Returns `None` if there is no child corresponding to token ATAT
fn ATAT(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_ATAT, 0)
}
/// Retrieves first TerminalNode corresponding to token LBRACE
/// Returns `None` if there is no child corresponding to token LBRACE
fn LBRACE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_LBRACE, 0)
}
/// Retrieves first TerminalNode corresponding to token RBRACE
/// Returns `None` if there is no child corresponding to token RBRACE
fn RBRACE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_RBRACE, 0)
}
/// Retrieves first TerminalNode corresponding to token AT
/// Returns `None` if there is no child corresponding to token AT
fn AT(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_AT, 0)
}
/// Retrieves first TerminalNode corresponding to token SEMI
/// Returns `None` if there is no child corresponding to token SEMI
fn SEMI(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_SEMI, 0)
}
/// Retrieves first TerminalNode corresponding to token COMMA
/// Returns `None` if there is no child corresponding to token COMMA
fn COMMA(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_COMMA, 0)
}
/// Retrieves first TerminalNode corresponding to token COLON
/// Returns `None` if there is no child corresponding to token COLON
fn COLON(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_COLON, 0)
}
/// Retrieves first TerminalNode corresponding to token STAR
/// Returns `None` if there is no child corresponding to token STAR
fn STAR(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_STAR, 0)
}
/// Retrieves first TerminalNode corresponding to token BANG
/// Returns `None` if there is no child corresponding to token BANG
fn BANG(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_BANG, 0)
}
/// Retrieves first TerminalNode corresponding to token PLUS
/// Returns `None` if there is no child corresponding to token PLUS
fn PLUS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_PLUS, 0)
}
/// Retrieves first TerminalNode corresponding to token MINUS
/// Returns `None` if there is no child corresponding to token MINUS
fn MINUS(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_MINUS, 0)
}
/// Retrieves first TerminalNode corresponding to token DOT
/// Returns `None` if there is no child corresponding to token DOT
fn DOT(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_DOT, 0)
}
/// Retrieves first TerminalNode corresponding to token OTHER
/// Returns `None` if there is no child corresponding to token OTHER
fn OTHER(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_OTHER, 0)
}

}

impl<'input> AnyTokenContextAttrs<'input> for AnyTokenContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn anyToken(&mut self,)
	-> Result<Rc<AnyTokenContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AnyTokenContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 72, RULE_anyToken);
        let mut _localctx: Rc<AnyTokenContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(406);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Ledger_ACCOUNT |Ledger_OPENING |Ledger_NAME |Ledger_PRICE_DIRECTIVE |
			Ledger_ALNUM 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule wordToken*/
					recog.base.set_state(386);
					recog.wordToken()?;

					}
				}

			Ledger_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(387);
					recog.base.match_token(Ledger_NUMBER,&mut recog.err_handler)?;

					}
				}

			Ledger_GROUPED_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(388);
					recog.base.match_token(Ledger_GROUPED_NUMBER,&mut recog.err_handler)?;

					}
				}

			Ledger_DATETIME 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(389);
					recog.base.match_token(Ledger_DATETIME,&mut recog.err_handler)?;

					}
				}

			Ledger_QUOTED 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					recog.base.set_state(390);
					recog.base.match_token(Ledger_QUOTED,&mut recog.err_handler)?;

					}
				}

			Ledger_DOUBLE_LBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					recog.base.set_state(391);
					recog.base.match_token(Ledger_DOUBLE_LBRACE,&mut recog.err_handler)?;

					}
				}

			Ledger_DOUBLE_RBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					recog.base.set_state(392);
					recog.base.match_token(Ledger_DOUBLE_RBRACE,&mut recog.err_handler)?;

					}
				}

			Ledger_ATAT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					recog.base.set_state(393);
					recog.base.match_token(Ledger_ATAT,&mut recog.err_handler)?;

					}
				}

			Ledger_LBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 9)?;
					recog.base.enter_outer_alt(None, 9)?;
					{
					recog.base.set_state(394);
					recog.base.match_token(Ledger_LBRACE,&mut recog.err_handler)?;

					}
				}

			Ledger_RBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 10)?;
					recog.base.enter_outer_alt(None, 10)?;
					{
					recog.base.set_state(395);
					recog.base.match_token(Ledger_RBRACE,&mut recog.err_handler)?;

					}
				}

			Ledger_AT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 11)?;
					recog.base.enter_outer_alt(None, 11)?;
					{
					recog.base.set_state(396);
					recog.base.match_token(Ledger_AT,&mut recog.err_handler)?;

					}
				}

			Ledger_SEMI 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 12)?;
					recog.base.enter_outer_alt(None, 12)?;
					{
					recog.base.set_state(397);
					recog.base.match_token(Ledger_SEMI,&mut recog.err_handler)?;

					}
				}

			Ledger_COMMA 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 13)?;
					recog.base.enter_outer_alt(None, 13)?;
					{
					recog.base.set_state(398);
					recog.base.match_token(Ledger_COMMA,&mut recog.err_handler)?;

					}
				}

			Ledger_COLON 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 14)?;
					recog.base.enter_outer_alt(None, 14)?;
					{
					recog.base.set_state(399);
					recog.base.match_token(Ledger_COLON,&mut recog.err_handler)?;

					}
				}

			Ledger_STAR 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 15)?;
					recog.base.enter_outer_alt(None, 15)?;
					{
					recog.base.set_state(400);
					recog.base.match_token(Ledger_STAR,&mut recog.err_handler)?;

					}
				}

			Ledger_BANG 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 16)?;
					recog.base.enter_outer_alt(None, 16)?;
					{
					recog.base.set_state(401);
					recog.base.match_token(Ledger_BANG,&mut recog.err_handler)?;

					}
				}

			Ledger_PLUS 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 17)?;
					recog.base.enter_outer_alt(None, 17)?;
					{
					recog.base.set_state(402);
					recog.base.match_token(Ledger_PLUS,&mut recog.err_handler)?;

					}
				}

			Ledger_MINUS 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 18)?;
					recog.base.enter_outer_alt(None, 18)?;
					{
					recog.base.set_state(403);
					recog.base.match_token(Ledger_MINUS,&mut recog.err_handler)?;

					}
				}

			Ledger_DOT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 19)?;
					recog.base.enter_outer_alt(None, 19)?;
					{
					recog.base.set_state(404);
					recog.base.match_token(Ledger_DOT,&mut recog.err_handler)?;

					}
				}

			Ledger_OTHER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 20)?;
					recog.base.enter_outer_alt(None, 20)?;
					{
					recog.base.set_state(405);
					recog.base.match_token(Ledger_OTHER,&mut recog.err_handler)?;

					}
				}

				_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
//------------------- lineEnd ----------------
pub type LineEndContextAll<'input> = LineEndContext<'input>;


pub type LineEndContext<'input> = BaseParserRuleContext<'input,LineEndContextExt<'input>>;

#[derive(Clone)]
pub struct LineEndContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> LedgerContext<'input> for LineEndContext<'input>{}

impl<'input,'a> Listenable<dyn LedgerNoOpListener<'input> + 'a> for LineEndContext<'input>{}

impl<'input,'a> Visitable<dyn LedgerVisitor<'input> + 'a> for LineEndContext<'input>{
	fn accept(&self,visitor: &mut (dyn LedgerVisitor<'input> + 'a)) {
		visitor.visit_lineEnd(self);
	}
}

impl<'input> CustomRuleContext<'input> for LineEndContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = LedgerContextType;
	fn get_rule_index(&self) -> usize { RULE_lineEnd }
	//fn type_rule_index() -> usize where Self: Sized { RULE_lineEnd }
}
antlr4rust::tid!{LineEndContextExt<'a>}

impl<'input> LineEndContextExt<'input>{
	fn new(parent: Option<Rc<dyn LedgerContext<'input> + 'input > >, invoking_state: i32) -> Rc<LineEndContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,LineEndContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait LineEndContextAttrs<'input>: LedgerContext<'input> + BorrowMut<LineEndContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token NEWLINE
/// Returns `None` if there is no child corresponding to token NEWLINE
fn NEWLINE(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_NEWLINE, 0)
}
/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,LedgerContextType>>> where Self:Sized{
	self.get_token(Ledger_EOF, 0)
}

}

impl<'input> LineEndContextAttrs<'input> for LineEndContext<'input>{}

impl<'input, I> Ledger<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn lineEnd(&mut self,)
	-> Result<Rc<LineEndContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = LineEndContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 74, RULE_lineEnd);
        let mut _localctx: Rc<LineEndContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(408);
			_la = recog.base.input.la(1);
			if { !(_la==Ledger_EOF || _la==Ledger_NEWLINE) } {
				recog.err_handler.recover_inline(&mut recog.base)?;

			}
			else {
				if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
				recog.err_handler.report_match(&mut recog.base);
				recog.base.consume(&mut recog.err_handler);
			}
			}
			Ok(())
		})();
		match result {
		Ok(_)=>{},
        Err(e @ ANTLRError::FallThrough(_)) => return Err(e),
		Err(ref re) => {
				//_localctx.exception = re;
				recog.err_handler.report_error(&mut recog.base, re);
				recog.err_handler.recover(&mut recog.base, re)?;
			}
		}
		recog.base.exit_rule()?;

		Ok(_localctx)
	}
}
	lazy_static!{
    static ref _ATN: Arc<ATN> =
        Arc::new(ATNDeserializer::new(None).deserialize(&mut _serializedATN.iter()));
    static ref _decision_to_DFA: Arc<Vec<antlr4rust::RwLock<DFA>>> = {
        let mut dfa = Vec::new();
        let size = _ATN.decision_to_state.len() as i32;
        for i in 0..size {
            dfa.push(DFA::new(
                _ATN.clone(),
                _ATN.get_decision_state(i),
                i,
            ).into())
        }
        Arc::new(dfa)
    };
	static ref _serializedATN: Vec<i32> = vec![
		4, 1, 26, 411, 2, 0, 7, 0, 2, 1, 7, 1, 2, 2, 7, 2, 2, 3, 7, 3, 2, 4, 7, 
		4, 2, 5, 7, 5, 2, 6, 7, 6, 2, 7, 7, 7, 2, 8, 7, 8, 2, 9, 7, 9, 2, 10, 
		7, 10, 2, 11, 7, 11, 2, 12, 7, 12, 2, 13, 7, 13, 2, 14, 7, 14, 2, 15, 
		7, 15, 2, 16, 7, 16, 2, 17, 7, 17, 2, 18, 7, 18, 2, 19, 7, 19, 2, 20, 
		7, 20, 2, 21, 7, 21, 2, 22, 7, 22, 2, 23, 7, 23, 2, 24, 7, 24, 2, 25, 
		7, 25, 2, 26, 7, 26, 2, 27, 7, 27, 2, 28, 7, 28, 2, 29, 7, 29, 2, 30, 
		7, 30, 2, 31, 7, 31, 2, 32, 7, 32, 2, 33, 7, 33, 2, 34, 7, 34, 2, 35, 
		7, 35, 2, 36, 7, 36, 2, 37, 7, 37, 1, 0, 5, 0, 78, 8, 0, 10, 0, 12, 0, 
		81, 9, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 89, 8, 1, 1, 2, 1, 
		2, 4, 2, 93, 8, 2, 11, 2, 12, 2, 94, 1, 2, 5, 2, 98, 8, 2, 10, 2, 12, 
		2, 101, 9, 2, 1, 3, 1, 3, 1, 3, 3, 3, 106, 8, 3, 1, 3, 5, 3, 109, 8, 3, 
		10, 3, 12, 3, 112, 9, 3, 1, 3, 1, 3, 1, 3, 5, 3, 117, 8, 3, 10, 3, 12, 
		3, 120, 9, 3, 1, 3, 5, 3, 123, 8, 3, 10, 3, 12, 3, 126, 9, 3, 1, 3, 1, 
		3, 1, 3, 1, 4, 1, 4, 1, 5, 1, 5, 3, 5, 135, 8, 5, 1, 6, 1, 6, 1, 6, 1, 
		6, 1, 6, 1, 6, 1, 6, 5, 6, 144, 8, 6, 10, 6, 12, 6, 147, 9, 6, 1, 6, 1, 
		6, 1, 7, 1, 7, 1, 7, 1, 7, 1, 7, 3, 7, 156, 8, 7, 1, 8, 1, 8, 5, 8, 160, 
		8, 8, 10, 8, 12, 8, 163, 9, 8, 1, 8, 5, 8, 166, 8, 8, 10, 8, 12, 8, 169, 
		9, 8, 1, 9, 1, 9, 1, 9, 1, 9, 1, 9, 3, 9, 176, 8, 9, 1, 9, 5, 9, 179, 
		8, 9, 10, 9, 12, 9, 182, 9, 9, 1, 9, 3, 9, 185, 8, 9, 1, 9, 1, 9, 1, 10, 
		1, 10, 1, 10, 3, 10, 192, 8, 10, 1, 10, 5, 10, 195, 8, 10, 10, 10, 12, 
		10, 198, 9, 10, 1, 10, 3, 10, 201, 8, 10, 1, 10, 1, 10, 1, 11, 1, 11, 
		1, 11, 1, 11, 1, 11, 3, 11, 210, 8, 11, 1, 12, 1, 12, 1, 12, 1, 12, 1, 
		13, 1, 13, 1, 13, 5, 13, 219, 8, 13, 10, 13, 12, 13, 222, 9, 13, 1, 14, 
		3, 14, 225, 8, 14, 1, 14, 1, 14, 1, 14, 1, 15, 3, 15, 231, 8, 15, 1, 15, 
		1, 15, 1, 16, 1, 16, 1, 16, 5, 16, 238, 8, 16, 10, 16, 12, 16, 241, 9, 
		16, 1, 17, 1, 17, 1, 17, 1, 18, 1, 18, 1, 18, 1, 19, 1, 19, 1, 19, 1, 
		20, 1, 20, 1, 20, 1, 20, 1, 20, 5, 20, 257, 8, 20, 10, 20, 12, 20, 260, 
		9, 20, 1, 21, 4, 21, 263, 8, 21, 11, 21, 12, 21, 264, 1, 22, 1, 22, 1, 
		22, 1, 22, 3, 22, 271, 8, 22, 1, 23, 1, 23, 5, 23, 275, 8, 23, 10, 23, 
		12, 23, 278, 9, 23, 1, 24, 1, 24, 3, 24, 282, 8, 24, 1, 25, 1, 25, 1, 
		25, 1, 25, 3, 25, 288, 8, 25, 1, 26, 3, 26, 291, 8, 26, 1, 26, 1, 26, 
		1, 27, 1, 27, 1, 27, 1, 27, 1, 27, 1, 27, 1, 27, 1, 27, 3, 27, 303, 8, 
		27, 1, 28, 5, 28, 306, 8, 28, 10, 28, 12, 28, 309, 9, 28, 1, 28, 3, 28, 
		312, 8, 28, 1, 28, 5, 28, 315, 8, 28, 10, 28, 12, 28, 318, 9, 28, 1, 28, 
		1, 28, 5, 28, 322, 8, 28, 10, 28, 12, 28, 325, 9, 28, 1, 28, 5, 28, 328, 
		8, 28, 10, 28, 12, 28, 331, 9, 28, 1, 28, 5, 28, 334, 8, 28, 10, 28, 12, 
		28, 337, 9, 28, 1, 29, 1, 29, 1, 29, 5, 29, 342, 8, 29, 10, 29, 12, 29, 
		345, 9, 29, 1, 30, 1, 30, 4, 30, 349, 8, 30, 11, 30, 12, 30, 350, 1, 31, 
		1, 31, 5, 31, 355, 8, 31, 10, 31, 12, 31, 358, 9, 31, 1, 31, 1, 31, 1, 
		31, 1, 31, 1, 32, 4, 32, 365, 8, 32, 11, 32, 12, 32, 366, 1, 33, 1, 33, 
		1, 33, 1, 33, 1, 33, 1, 33, 1, 33, 1, 33, 1, 33, 3, 33, 378, 8, 33, 1, 
		34, 1, 34, 1, 35, 1, 35, 1, 35, 3, 35, 385, 8, 35, 1, 36, 1, 36, 1, 36, 
		1, 36, 1, 36, 1, 36, 1, 36, 1, 36, 1, 36, 1, 36, 1, 36, 1, 36, 1, 36, 
		1, 36, 1, 36, 1, 36, 1, 36, 1, 36, 1, 36, 1, 36, 3, 36, 407, 8, 36, 1, 
		37, 1, 37, 1, 37, 0, 0, 38, 0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 
		24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 
		60, 62, 64, 66, 68, 70, 72, 74, 0, 6, 1, 0, 14, 15, 1, 0, 16, 17, 1, 0, 
		20, 21, 2, 0, 7, 7, 10, 10, 2, 0, 1, 4, 23, 23, 1, 1, 25, 25, 454, 0, 
		79, 1, 0, 0, 0, 2, 88, 1, 0, 0, 0, 4, 90, 1, 0, 0, 0, 6, 102, 1, 0, 0, 
		0, 8, 130, 1, 0, 0, 0, 10, 134, 1, 0, 0, 0, 12, 136, 1, 0, 0, 0, 14, 155, 
		1, 0, 0, 0, 16, 157, 1, 0, 0, 0, 18, 170, 1, 0, 0, 0, 20, 188, 1, 0, 0, 
		0, 22, 204, 1, 0, 0, 0, 24, 211, 1, 0, 0, 0, 26, 215, 1, 0, 0, 0, 28, 
		224, 1, 0, 0, 0, 30, 230, 1, 0, 0, 0, 32, 234, 1, 0, 0, 0, 34, 242, 1, 
		0, 0, 0, 36, 245, 1, 0, 0, 0, 38, 248, 1, 0, 0, 0, 40, 251, 1, 0, 0, 0, 
		42, 262, 1, 0, 0, 0, 44, 270, 1, 0, 0, 0, 46, 272, 1, 0, 0, 0, 48, 281, 
		1, 0, 0, 0, 50, 287, 1, 0, 0, 0, 52, 290, 1, 0, 0, 0, 54, 302, 1, 0, 0, 
		0, 56, 307, 1, 0, 0, 0, 58, 338, 1, 0, 0, 0, 60, 348, 1, 0, 0, 0, 62, 
		352, 1, 0, 0, 0, 64, 364, 1, 0, 0, 0, 66, 377, 1, 0, 0, 0, 68, 379, 1, 
		0, 0, 0, 70, 384, 1, 0, 0, 0, 72, 406, 1, 0, 0, 0, 74, 408, 1, 0, 0, 0, 
		76, 78, 3, 2, 1, 0, 77, 76, 1, 0, 0, 0, 78, 81, 1, 0, 0, 0, 79, 77, 1, 
		0, 0, 0, 79, 80, 1, 0, 0, 0, 80, 82, 1, 0, 0, 0, 81, 79, 1, 0, 0, 0, 82, 
		83, 5, 0, 0, 1, 83, 1, 1, 0, 0, 0, 84, 89, 3, 30, 15, 0, 85, 89, 3, 28, 
		14, 0, 86, 89, 3, 16, 8, 0, 87, 89, 3, 4, 2, 0, 88, 84, 1, 0, 0, 0, 88, 
		85, 1, 0, 0, 0, 88, 86, 1, 0, 0, 0, 88, 87, 1, 0, 0, 0, 89, 3, 1, 0, 0, 
		0, 90, 92, 3, 6, 3, 0, 91, 93, 3, 12, 6, 0, 92, 91, 1, 0, 0, 0, 93, 94, 
		1, 0, 0, 0, 94, 92, 1, 0, 0, 0, 94, 95, 1, 0, 0, 0, 95, 99, 1, 0, 0, 0, 
		96, 98, 3, 30, 15, 0, 97, 96, 1, 0, 0, 0, 98, 101, 1, 0, 0, 0, 99, 97, 
		1, 0, 0, 0, 99, 100, 1, 0, 0, 0, 100, 5, 1, 0, 0, 0, 101, 99, 1, 0, 0, 
		0, 102, 103, 5, 19, 0, 0, 103, 105, 5, 24, 0, 0, 104, 106, 3, 8, 4, 0, 
		105, 104, 1, 0, 0, 0, 105, 106, 1, 0, 0, 0, 106, 110, 1, 0, 0, 0, 107, 
		109, 5, 24, 0, 0, 108, 107, 1, 0, 0, 0, 109, 112, 1, 0, 0, 0, 110, 108, 
		1, 0, 0, 0, 110, 111, 1, 0, 0, 0, 111, 113, 1, 0, 0, 0, 112, 110, 1, 0, 
		0, 0, 113, 118, 3, 10, 5, 0, 114, 115, 5, 24, 0, 0, 115, 117, 3, 10, 5, 
		0, 116, 114, 1, 0, 0, 0, 117, 120, 1, 0, 0, 0, 118, 116, 1, 0, 0, 0, 118, 
		119, 1, 0, 0, 0, 119, 124, 1, 0, 0, 0, 120, 118, 1, 0, 0, 0, 121, 123, 
		5, 24, 0, 0, 122, 121, 1, 0, 0, 0, 123, 126, 1, 0, 0, 0, 124, 122, 1, 
		0, 0, 0, 124, 125, 1, 0, 0, 0, 125, 127, 1, 0, 0, 0, 126, 124, 1, 0, 0, 
		0, 127, 128, 3, 32, 16, 0, 128, 129, 3, 74, 37, 0, 129, 7, 1, 0, 0, 0, 
		130, 131, 7, 0, 0, 0, 131, 9, 1, 0, 0, 0, 132, 135, 5, 22, 0, 0, 133, 
		135, 3, 64, 32, 0, 134, 132, 1, 0, 0, 0, 134, 133, 1, 0, 0, 0, 135, 11, 
		1, 0, 0, 0, 136, 137, 5, 24, 0, 0, 137, 138, 3, 40, 20, 0, 138, 139, 5, 
		24, 0, 0, 139, 140, 3, 52, 26, 0, 140, 141, 5, 24, 0, 0, 141, 145, 3, 
		46, 23, 0, 142, 144, 3, 14, 7, 0, 143, 142, 1, 0, 0, 0, 144, 147, 1, 0, 
		0, 0, 145, 143, 1, 0, 0, 0, 145, 146, 1, 0, 0, 0, 146, 148, 1, 0, 0, 0, 
		147, 145, 1, 0, 0, 0, 148, 149, 3, 74, 37, 0, 149, 13, 1, 0, 0, 0, 150, 
		156, 5, 24, 0, 0, 151, 156, 3, 54, 27, 0, 152, 156, 3, 62, 31, 0, 153, 
		156, 3, 32, 16, 0, 154, 156, 3, 70, 35, 0, 155, 150, 1, 0, 0, 0, 155, 
		151, 1, 0, 0, 0, 155, 152, 1, 0, 0, 0, 155, 153, 1, 0, 0, 0, 155, 154, 
		1, 0, 0, 0, 156, 15, 1, 0, 0, 0, 157, 161, 3, 18, 9, 0, 158, 160, 3, 20, 
		10, 0, 159, 158, 1, 0, 0, 0, 160, 163, 1, 0, 0, 0, 161, 159, 1, 0, 0, 
		0, 161, 162, 1, 0, 0, 0, 162, 167, 1, 0, 0, 0, 163, 161, 1, 0, 0, 0, 164, 
		166, 3, 30, 15, 0, 165, 164, 1, 0, 0, 0, 166, 169, 1, 0, 0, 0, 167, 165, 
		1, 0, 0, 0, 167, 168, 1, 0, 0, 0, 168, 17, 1, 0, 0, 0, 169, 167, 1, 0, 
		0, 0, 170, 171, 5, 1, 0, 0, 171, 172, 5, 24, 0, 0, 172, 175, 3, 40, 20, 
		0, 173, 174, 5, 24, 0, 0, 174, 176, 3, 46, 23, 0, 175, 173, 1, 0, 0, 0, 
		175, 176, 1, 0, 0, 0, 176, 180, 1, 0, 0, 0, 177, 179, 5, 24, 0, 0, 178, 
		177, 1, 0, 0, 0, 179, 182, 1, 0, 0, 0, 180, 178, 1, 0, 0, 0, 180, 181, 
		1, 0, 0, 0, 181, 184, 1, 0, 0, 0, 182, 180, 1, 0, 0, 0, 183, 185, 3, 32, 
		16, 0, 184, 183, 1, 0, 0, 0, 184, 185, 1, 0, 0, 0, 185, 186, 1, 0, 0, 
		0, 186, 187, 3, 74, 37, 0, 187, 19, 1, 0, 0, 0, 188, 191, 5, 24, 0, 0, 
		189, 192, 3, 22, 11, 0, 190, 192, 3, 24, 12, 0, 191, 189, 1, 0, 0, 0, 
		191, 190, 1, 0, 0, 0, 192, 196, 1, 0, 0, 0, 193, 195, 5, 24, 0, 0, 194, 
		193, 1, 0, 0, 0, 195, 198, 1, 0, 0, 0, 196, 194, 1, 0, 0, 0, 196, 197, 
		1, 0, 0, 0, 197, 200, 1, 0, 0, 0, 198, 196, 1, 0, 0, 0, 199, 201, 3, 32, 
		16, 0, 200, 199, 1, 0, 0, 0, 200, 201, 1, 0, 0, 0, 201, 202, 1, 0, 0, 
		0, 202, 203, 3, 74, 37, 0, 203, 21, 1, 0, 0, 0, 204, 205, 5, 2, 0, 0, 
		205, 206, 5, 24, 0, 0, 206, 209, 3, 52, 26, 0, 207, 208, 5, 24, 0, 0, 
		208, 210, 3, 46, 23, 0, 209, 207, 1, 0, 0, 0, 209, 210, 1, 0, 0, 0, 210, 
		23, 1, 0, 0, 0, 211, 212, 5, 3, 0, 0, 212, 213, 5, 24, 0, 0, 213, 214, 
		3, 26, 13, 0, 214, 25, 1, 0, 0, 0, 215, 220, 3, 10, 5, 0, 216, 217, 5, 
		24, 0, 0, 217, 219, 3, 10, 5, 0, 218, 216, 1, 0, 0, 0, 219, 222, 1, 0, 
		0, 0, 220, 218, 1, 0, 0, 0, 220, 221, 1, 0, 0, 0, 221, 27, 1, 0, 0, 0, 
		222, 220, 1, 0, 0, 0, 223, 225, 5, 24, 0, 0, 224, 223, 1, 0, 0, 0, 224, 
		225, 1, 0, 0, 0, 225, 226, 1, 0, 0, 0, 226, 227, 3, 32, 16, 0, 227, 228, 
		3, 74, 37, 0, 228, 29, 1, 0, 0, 0, 229, 231, 5, 24, 0, 0, 230, 229, 1, 
		0, 0, 0, 230, 231, 1, 0, 0, 0, 231, 232, 1, 0, 0, 0, 232, 233, 5, 25, 
		0, 0, 233, 31, 1, 0, 0, 0, 234, 239, 5, 11, 0, 0, 235, 238, 5, 24, 0, 
		0, 236, 238, 3, 72, 36, 0, 237, 235, 1, 0, 0, 0, 237, 236, 1, 0, 0, 0, 
		238, 241, 1, 0, 0, 0, 239, 237, 1, 0, 0, 0, 239, 240, 1, 0, 0, 0, 240, 
		33, 1, 0, 0, 0, 241, 239, 1, 0, 0, 0, 242, 243, 3, 40, 20, 0, 243, 244, 
		5, 0, 0, 1, 244, 35, 1, 0, 0, 0, 245, 246, 3, 46, 23, 0, 246, 247, 5, 
		0, 0, 1, 247, 37, 1, 0, 0, 0, 248, 249, 3, 52, 26, 0, 249, 250, 5, 0, 
		0, 1, 250, 39, 1, 0, 0, 0, 251, 252, 3, 42, 21, 0, 252, 253, 5, 13, 0, 
		0, 253, 258, 3, 42, 21, 0, 254, 255, 5, 13, 0, 0, 255, 257, 3, 42, 21, 
		0, 256, 254, 1, 0, 0, 0, 257, 260, 1, 0, 0, 0, 258, 256, 1, 0, 0, 0, 258, 
		259, 1, 0, 0, 0, 259, 41, 1, 0, 0, 0, 260, 258, 1, 0, 0, 0, 261, 263, 
		3, 44, 22, 0, 262, 261, 1, 0, 0, 0, 263, 264, 1, 0, 0, 0, 264, 262, 1, 
		0, 0, 0, 264, 265, 1, 0, 0, 0, 265, 43, 1, 0, 0, 0, 266, 271, 3, 68, 34, 
		0, 267, 271, 5, 21, 0, 0, 268, 271, 5, 18, 0, 0, 269, 271, 5, 17, 0, 0, 
		270, 266, 1, 0, 0, 0, 270, 267, 1, 0, 0, 0, 270, 268, 1, 0, 0, 0, 270, 
		269, 1, 0, 0, 0, 271, 45, 1, 0, 0, 0, 272, 276, 3, 48, 24, 0, 273, 275, 
		3, 50, 25, 0, 274, 273, 1, 0, 0, 0, 275, 278, 1, 0, 0, 0, 276, 274, 1, 
		0, 0, 0, 276, 277, 1, 0, 0, 0, 277, 47, 1, 0, 0, 0, 278, 276, 1, 0, 0, 
		0, 279, 282, 3, 68, 34, 0, 280, 282, 5, 21, 0, 0, 281, 279, 1, 0, 0, 0, 
		281, 280, 1, 0, 0, 0, 282, 49, 1, 0, 0, 0, 283, 288, 3, 48, 24, 0, 284, 
		288, 5, 18, 0, 0, 285, 288, 5, 17, 0, 0, 286, 288, 5, 13, 0, 0, 287, 283, 
		1, 0, 0, 0, 287, 284, 1, 0, 0, 0, 287, 285, 1, 0, 0, 0, 287, 286, 1, 0, 
		0, 0, 288, 51, 1, 0, 0, 0, 289, 291, 7, 1, 0, 0, 290, 289, 1, 0, 0, 0, 
		290, 291, 1, 0, 0, 0, 291, 292, 1, 0, 0, 0, 292, 293, 7, 2, 0, 0, 293, 
		53, 1, 0, 0, 0, 294, 295, 5, 8, 0, 0, 295, 296, 3, 56, 28, 0, 296, 297, 
		5, 9, 0, 0, 297, 303, 1, 0, 0, 0, 298, 299, 5, 5, 0, 0, 299, 300, 3, 56, 
		28, 0, 300, 301, 5, 6, 0, 0, 301, 303, 1, 0, 0, 0, 302, 294, 1, 0, 0, 
		0, 302, 298, 1, 0, 0, 0, 303, 55, 1, 0, 0, 0, 304, 306, 5, 24, 0, 0, 305, 
		304, 1, 0, 0, 0, 306, 309, 1, 0, 0, 0, 307, 305, 1, 0, 0, 0, 307, 308, 
		1, 0, 0, 0, 308, 311, 1, 0, 0, 0, 309, 307, 1, 0, 0, 0, 310, 312, 3, 58, 
		29, 0, 311, 310, 1, 0, 0, 0, 311, 312, 1, 0, 0, 0, 312, 329, 1, 0, 0, 
		0, 313, 315, 5, 24, 0, 0, 314, 313, 1, 0, 0, 0, 315, 318, 1, 0, 0, 0, 
		316, 314, 1, 0, 0, 0, 316, 317, 1, 0, 0, 0, 317, 319, 1, 0, 0, 0, 318, 
		316, 1, 0, 0, 0, 319, 323, 5, 12, 0, 0, 320, 322, 5, 24, 0, 0, 321, 320, 
		1, 0, 0, 0, 322, 325, 1, 0, 0, 0, 323, 321, 1, 0, 0, 0, 323, 324, 1, 0, 
		0, 0, 324, 326, 1, 0, 0, 0, 325, 323, 1, 0, 0, 0, 326, 328, 3, 58, 29, 
		0, 327, 316, 1, 0, 0, 0, 328, 331, 1, 0, 0, 0, 329, 327, 1, 0, 0, 0, 329, 
		330, 1, 0, 0, 0, 330, 335, 1, 0, 0, 0, 331, 329, 1, 0, 0, 0, 332, 334, 
		5, 24, 0, 0, 333, 332, 1, 0, 0, 0, 334, 337, 1, 0, 0, 0, 335, 333, 1, 
		0, 0, 0, 335, 336, 1, 0, 0, 0, 336, 57, 1, 0, 0, 0, 337, 335, 1, 0, 0, 
		0, 338, 343, 3, 60, 30, 0, 339, 340, 5, 24, 0, 0, 340, 342, 3, 60, 30, 
		0, 341, 339, 1, 0, 0, 0, 342, 345, 1, 0, 0, 0, 343, 341, 1, 0, 0, 0, 343, 
		344, 1, 0, 0, 0, 344, 59, 1, 0, 0, 0, 345, 343, 1, 0, 0, 0, 346, 349, 
		5, 22, 0, 0, 347, 349, 3, 66, 33, 0, 348, 346, 1, 0, 0, 0, 348, 347, 1, 
		0, 0, 0, 349, 350, 1, 0, 0, 0, 350, 348, 1, 0, 0, 0, 350, 351, 1, 0, 0, 
		0, 351, 61, 1, 0, 0, 0, 352, 356, 7, 3, 0, 0, 353, 355, 5, 24, 0, 0, 354, 
		353, 1, 0, 0, 0, 355, 358, 1, 0, 0, 0, 356, 354, 1, 0, 0, 0, 356, 357, 
		1, 0, 0, 0, 357, 359, 1, 0, 0, 0, 358, 356, 1, 0, 0, 0, 359, 360, 3, 52, 
		26, 0, 360, 361, 5, 24, 0, 0, 361, 362, 3, 46, 23, 0, 362, 63, 1, 0, 0, 
		0, 363, 365, 3, 66, 33, 0, 364, 363, 1, 0, 0, 0, 365, 366, 1, 0, 0, 0, 
		366, 364, 1, 0, 0, 0, 366, 367, 1, 0, 0, 0, 367, 65, 1, 0, 0, 0, 368, 
		378, 3, 68, 34, 0, 369, 378, 5, 21, 0, 0, 370, 378, 5, 20, 0, 0, 371, 
		378, 5, 19, 0, 0, 372, 378, 5, 18, 0, 0, 373, 378, 5, 17, 0, 0, 374, 378, 
		5, 16, 0, 0, 375, 378, 5, 13, 0, 0, 376, 378, 5, 26, 0, 0, 377, 368, 1, 
		0, 0, 0, 377, 369, 1, 0, 0, 0, 377, 370, 1, 0, 0, 0, 377, 371, 1, 0, 0, 
		0, 377, 372, 1, 0, 0, 0, 377, 373, 1, 0, 0, 0, 377, 374, 1, 0, 0, 0, 377, 
		375, 1, 0, 0, 0, 377, 376, 1, 0, 0, 0, 378, 67, 1, 0, 0, 0, 379, 380, 
		7, 4, 0, 0, 380, 69, 1, 0, 0, 0, 381, 385, 5, 22, 0, 0, 382, 385, 5, 12, 
		0, 0, 383, 385, 3, 64, 32, 0, 384, 381, 1, 0, 0, 0, 384, 382, 1, 0, 0, 
		0, 384, 383, 1, 0, 0, 0, 385, 71, 1, 0, 0, 0, 386, 407, 3, 68, 34, 0, 
		387, 407, 5, 21, 0, 0, 388, 407, 5, 20, 0, 0, 389, 407, 5, 19, 0, 0, 390, 
		407, 5, 22, 0, 0, 391, 407, 5, 5, 0, 0, 392, 407, 5, 6, 0, 0, 393, 407, 
		5, 7, 0, 0, 394, 407, 5, 8, 0, 0, 395, 407, 5, 9, 0, 0, 396, 407, 5, 10, 
		0, 0, 397, 407, 5, 11, 0, 0, 398, 407, 5, 12, 0, 0, 399, 407, 5, 13, 0, 
		0, 400, 407, 5, 14, 0, 0, 401, 407, 5, 15, 0, 0, 402, 407, 5, 16, 0, 0, 
		403, 407, 5, 17, 0, 0, 404, 407, 5, 18, 0, 0, 405, 407, 5, 26, 0, 0, 406, 
		386, 1, 0, 0, 0, 406, 387, 1, 0, 0, 0, 406, 388, 1, 0, 0, 0, 406, 389, 
		1, 0, 0, 0, 406, 390, 1, 0, 0, 0, 406, 391, 1, 0, 0, 0, 406, 392, 1, 0, 
		0, 0, 406, 393, 1, 0, 0, 0, 406, 394, 1, 0, 0, 0, 406, 395, 1, 0, 0, 0, 
		406, 396, 1, 0, 0, 0, 406, 397, 1, 0, 0, 0, 406, 398, 1, 0, 0, 0, 406, 
		399, 1, 0, 0, 0, 406, 400, 1, 0, 0, 0, 406, 401, 1, 0, 0, 0, 406, 402, 
		1, 0, 0, 0, 406, 403, 1, 0, 0, 0, 406, 404, 1, 0, 0, 0, 406, 405, 1, 0, 
		0, 0, 407, 73, 1, 0, 0, 0, 408, 409, 7, 5, 0, 0, 409, 75, 1, 0, 0, 0, 
		47, 79, 88, 94, 99, 105, 110, 118, 124, 134, 145, 155, 161, 167, 175, 
		180, 184, 191, 196, 200, 209, 220, 224, 230, 237, 239, 258, 264, 270, 
		276, 281, 287, 290, 302, 307, 311, 316, 323, 329, 335, 343, 348, 350, 
		356, 366, 377, 384, 406
	];
}
