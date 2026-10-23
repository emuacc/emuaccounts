// Generated from Prices.g4 by ANTLR 4.13.2
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
use super::pricesvisitor::*;

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

		pub const Prices_ACCOUNT:i32=1; 
		pub const Prices_OPENING:i32=2; 
		pub const Prices_NAME:i32=3; 
		pub const Prices_PRICE_DIRECTIVE:i32=4; 
		pub const Prices_DOUBLE_LBRACE:i32=5; 
		pub const Prices_DOUBLE_RBRACE:i32=6; 
		pub const Prices_ATAT:i32=7; 
		pub const Prices_LBRACE:i32=8; 
		pub const Prices_RBRACE:i32=9; 
		pub const Prices_AT:i32=10; 
		pub const Prices_SEMI:i32=11; 
		pub const Prices_COMMA:i32=12; 
		pub const Prices_COLON:i32=13; 
		pub const Prices_STAR:i32=14; 
		pub const Prices_BANG:i32=15; 
		pub const Prices_PLUS:i32=16; 
		pub const Prices_MINUS:i32=17; 
		pub const Prices_DOT:i32=18; 
		pub const Prices_DATETIME:i32=19; 
		pub const Prices_GROUPED_NUMBER:i32=20; 
		pub const Prices_NUMBER:i32=21; 
		pub const Prices_QUOTED:i32=22; 
		pub const Prices_ALNUM:i32=23; 
		pub const Prices_WS:i32=24; 
		pub const Prices_NEWLINE:i32=25; 
		pub const Prices_OTHER:i32=26;
	pub const Prices_EOF:i32=EOF;
	pub const RULE_pricesFile:usize = 0; 
	pub const RULE_priceRecord:usize = 1; 
	pub const RULE_priceDirective:usize = 2; 
	pub const RULE_directive:usize = 3; 
	pub const RULE_blankLine:usize = 4; 
	pub const RULE_commodityInput:usize = 5; 
	pub const RULE_amountInput:usize = 6; 
	pub const RULE_commodity:usize = 7; 
	pub const RULE_commodityHead:usize = 8; 
	pub const RULE_commodityTail:usize = 9; 
	pub const RULE_amount:usize = 10; 
	pub const RULE_wordToken:usize = 11; 
	pub const RULE_anyToken:usize = 12; 
	pub const RULE_lineEnd:usize = 13;
	pub const ruleNames: [&'static str; 14] =  [
		"pricesFile", "priceRecord", "priceDirective", "directive", "blankLine", 
		"commodityInput", "amountInput", "commodity", "commodityHead", "commodityTail", 
		"amount", "wordToken", "anyToken", "lineEnd"
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
pub trait PricesNoOpListener<'input>: ParseTreeListener<'input,PricesContextType> {}
antlr4rust::coerce_from!{ 'input : PricesNoOpListener<'input> }
antlr4rust::tid! { impl<'input> TidAble<'input> for dyn PricesNoOpListener<'input> + 'input }

type BaseParserType<'input, I> =
	BaseParser<'input,PricesExt<'input>, I, PricesContextType, dyn PricesNoOpListener<'input> + 'input >;

type TokenType<'input> = <LocalTokenFactory<'input> as TokenFactory<'input>>::Tok;
pub type LocalTokenFactory<'input> = CommonTokenFactory;

pub type PricesTreeWalker<'input,'a> =
	ParseTreeWalker<'input, 'a, PricesContextType, dyn PricesNoOpListener<'input> + 'a >;

/// Parser for Prices grammar
pub struct Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	base:BaseParserType<'input,I>,
	interpreter:Arc<ParserATNSimulator>,
	_shared_context_cache: Box<PredictionContextCache>,
    pub err_handler: Box<dyn ErrorStrategy<'input,BaseParserType<'input,I> > >,
}

impl<'input, I> Prices<'input, I>
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
				PricesExt{
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

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn with_dyn_strategy(input: I) -> Self{
    	Self::with_strategy(input,Box::new(DefaultErrorStrategy::new()))
    }
}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    pub fn new(input: I) -> Self{
    	Self::with_strategy(input,Box::new(DefaultErrorStrategy::new()))
    }
}

/// Trait for monomorphized trait object that corresponds to the nodes of parse tree generated for Prices
pub trait PricesContext<'input>:
	for<'x> Visitable<dyn PricesVisitor<'input> + 'x > + 
	for<'x> Listenable<dyn PricesNoOpListener<'input> + 'x > + 
	ParserRuleContext<'input, TF=LocalTokenFactory<'input>, Ctx=PricesContextType>
{}

antlr4rust::coerce_from!{ 'input : PricesContext<'input> }

impl<'input, 'x, T> VisitableDyn<T> for dyn PricesContext<'input> + 'input
where
    T: PricesVisitor<'input> + 'x,
{
    fn accept_dyn(&self, visitor: &mut T) {
        self.accept(visitor as &mut (dyn PricesVisitor<'input> + 'x))
    }
}

impl<'input> PricesContext<'input> for TerminalNode<'input,PricesContextType> {}
impl<'input> PricesContext<'input> for ErrorNode<'input,PricesContextType> {}

antlr4rust::tid! { impl<'input> TidAble<'input> for dyn PricesContext<'input> + 'input }


pub struct PricesContextType;
antlr4rust::tid!{PricesContextType}

impl<'input> ParserNodeType<'input> for PricesContextType{
	type TF = LocalTokenFactory<'input>;
	type Type = dyn PricesContext<'input> + 'input;
}

impl<'input, I> Deref for Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    type Target = BaseParserType<'input,I>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'input, I> DerefMut for Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub struct PricesExt<'input>{
	_pd: PhantomData<&'input str>,
}

impl<'input> PricesExt<'input>{
}
antlr4rust::tid! { PricesExt<'a> }

impl<'input> TokenAware<'input> for PricesExt<'input>{
	type TF = LocalTokenFactory<'input>;
}

impl<'input,I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>> ParserRecog<'input, BaseParserType<'input,I>> for PricesExt<'input>{}

impl<'input,I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>> Actions<'input, BaseParserType<'input,I>> for PricesExt<'input>{
	fn get_grammar_file_name(&self) -> & str{ "Prices.g4"}

   	fn get_rule_names(&self) -> &[& str] {&ruleNames}

   	fn get_vocabulary(&self) -> &dyn Vocabulary { &**VOCABULARY }
}
//------------------- pricesFile ----------------
pub type PricesFileContextAll<'input> = PricesFileContext<'input>;


pub type PricesFileContext<'input> = BaseParserRuleContext<'input,PricesFileContextExt<'input>>;

#[derive(Clone)]
pub struct PricesFileContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> PricesContext<'input> for PricesFileContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for PricesFileContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for PricesFileContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_pricesFile(self);
	}
}

impl<'input> CustomRuleContext<'input> for PricesFileContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_pricesFile }
	//fn type_rule_index() -> usize where Self: Sized { RULE_pricesFile }
}
antlr4rust::tid!{PricesFileContextExt<'a>}

impl<'input> PricesFileContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<PricesFileContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PricesFileContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PricesFileContextAttrs<'input>: PricesContext<'input> + BorrowMut<PricesFileContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_EOF, 0)
}
fn priceRecord_all(&self) ->  Vec<Rc<PriceRecordContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn priceRecord(&self, i: usize) -> Option<Rc<PriceRecordContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> PricesFileContextAttrs<'input> for PricesFileContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn pricesFile(&mut self,)
	-> Result<Rc<PricesFileContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PricesFileContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 0, RULE_pricesFile);
        let mut _localctx: Rc<PricesFileContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(31);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 50333712) != 0) {
				{
				{
				/*InvokeRule priceRecord*/
				recog.base.set_state(28);
				recog.priceRecord()?;

				}
				}
				recog.base.set_state(33);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			recog.base.set_state(34);
			recog.base.match_token(Prices_EOF,&mut recog.err_handler)?;

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
//------------------- priceRecord ----------------
pub type PriceRecordContextAll<'input> = PriceRecordContext<'input>;


pub type PriceRecordContext<'input> = BaseParserRuleContext<'input,PriceRecordContextExt<'input>>;

#[derive(Clone)]
pub struct PriceRecordContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> PricesContext<'input> for PriceRecordContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for PriceRecordContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for PriceRecordContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_priceRecord(self);
	}
}

impl<'input> CustomRuleContext<'input> for PriceRecordContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_priceRecord }
	//fn type_rule_index() -> usize where Self: Sized { RULE_priceRecord }
}
antlr4rust::tid!{PriceRecordContextExt<'a>}

impl<'input> PriceRecordContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<PriceRecordContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PriceRecordContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PriceRecordContextAttrs<'input>: PricesContext<'input> + BorrowMut<PriceRecordContextExt<'input>>{

fn blankLine(&self) -> Option<Rc<BlankLineContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn directive(&self) -> Option<Rc<DirectiveContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn priceDirective(&self) -> Option<Rc<PriceDirectiveContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> PriceRecordContextAttrs<'input> for PriceRecordContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn priceRecord(&mut self,)
	-> Result<Rc<PriceRecordContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PriceRecordContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 2, RULE_priceRecord);
        let mut _localctx: Rc<PriceRecordContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(39);
			recog.err_handler.sync(&mut recog.base)?;
			match  recog.interpreter.adaptive_predict(1,&mut recog.base)? {
				1 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule blankLine*/
					recog.base.set_state(36);
					recog.blankLine()?;

					}
				}
			,
				2 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					/*InvokeRule directive*/
					recog.base.set_state(37);
					recog.directive()?;

					}
				}
			,
				3 =>{
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					/*InvokeRule priceDirective*/
					recog.base.set_state(38);
					recog.priceDirective()?;

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
//------------------- priceDirective ----------------
pub type PriceDirectiveContextAll<'input> = PriceDirectiveContext<'input>;


pub type PriceDirectiveContext<'input> = BaseParserRuleContext<'input,PriceDirectiveContextExt<'input>>;

#[derive(Clone)]
pub struct PriceDirectiveContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> PricesContext<'input> for PriceDirectiveContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for PriceDirectiveContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for PriceDirectiveContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_priceDirective(self);
	}
}

impl<'input> CustomRuleContext<'input> for PriceDirectiveContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_priceDirective }
	//fn type_rule_index() -> usize where Self: Sized { RULE_priceDirective }
}
antlr4rust::tid!{PriceDirectiveContextExt<'a>}

impl<'input> PriceDirectiveContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<PriceDirectiveContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,PriceDirectiveContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait PriceDirectiveContextAttrs<'input>: PricesContext<'input> + BorrowMut<PriceDirectiveContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token PRICE_DIRECTIVE
/// Returns `None` if there is no child corresponding to token PRICE_DIRECTIVE
fn PRICE_DIRECTIVE(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_PRICE_DIRECTIVE, 0)
}
/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,PricesContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_WS, i)
}
/// Retrieves first TerminalNode corresponding to token DATETIME
/// Returns `None` if there is no child corresponding to token DATETIME
fn DATETIME(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_DATETIME, 0)
}
fn commodity_all(&self) ->  Vec<Rc<CommodityContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn commodity(&self, i: usize) -> Option<Rc<CommodityContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}
fn amount(&self) -> Option<Rc<AmountContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}

}

impl<'input> PriceDirectiveContextAttrs<'input> for PriceDirectiveContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn priceDirective(&mut self,)
	-> Result<Rc<PriceDirectiveContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = PriceDirectiveContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 4, RULE_priceDirective);
        let mut _localctx: Rc<PriceDirectiveContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(41);
			recog.base.match_token(Prices_PRICE_DIRECTIVE,&mut recog.err_handler)?;

			recog.base.set_state(42);
			recog.base.match_token(Prices_WS,&mut recog.err_handler)?;

			recog.base.set_state(43);
			recog.base.match_token(Prices_DATETIME,&mut recog.err_handler)?;

			recog.base.set_state(44);
			recog.base.match_token(Prices_WS,&mut recog.err_handler)?;

			/*InvokeRule commodity*/
			recog.base.set_state(45);
			recog.commodity()?;

			recog.base.set_state(46);
			recog.base.match_token(Prices_WS,&mut recog.err_handler)?;

			/*InvokeRule amount*/
			recog.base.set_state(47);
			recog.amount()?;

			recog.base.set_state(48);
			recog.base.match_token(Prices_WS,&mut recog.err_handler)?;

			/*InvokeRule commodity*/
			recog.base.set_state(49);
			recog.commodity()?;

			recog.base.set_state(53);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while _la==Prices_WS {
				{
				{
				recog.base.set_state(50);
				recog.base.match_token(Prices_WS,&mut recog.err_handler)?;

				}
				}
				recog.base.set_state(55);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule lineEnd*/
			recog.base.set_state(56);
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
//------------------- directive ----------------
pub type DirectiveContextAll<'input> = DirectiveContext<'input>;


pub type DirectiveContext<'input> = BaseParserRuleContext<'input,DirectiveContextExt<'input>>;

#[derive(Clone)]
pub struct DirectiveContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> PricesContext<'input> for DirectiveContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for DirectiveContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for DirectiveContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_directive(self);
	}
}

impl<'input> CustomRuleContext<'input> for DirectiveContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_directive }
	//fn type_rule_index() -> usize where Self: Sized { RULE_directive }
}
antlr4rust::tid!{DirectiveContextExt<'a>}

impl<'input> DirectiveContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<DirectiveContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,DirectiveContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait DirectiveContextAttrs<'input>: PricesContext<'input> + BorrowMut<DirectiveContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token SEMI
/// Returns `None` if there is no child corresponding to token SEMI
fn SEMI(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_SEMI, 0)
}
fn lineEnd(&self) -> Option<Rc<LineEndContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves all `TerminalNode`s corresponding to token WS in current rule
fn WS_all(&self) -> Vec<Rc<TerminalNode<'input,PricesContextType>>>  where Self:Sized{
	self.children_of_type()
}
/// Retrieves 'i's TerminalNode corresponding to token WS, starting from 0.
/// Returns `None` if number of children corresponding to token WS is less or equal than `i`.
fn WS(&self, i: usize) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_WS, i)
}
fn anyToken_all(&self) ->  Vec<Rc<AnyTokenContextAll<'input>>> where Self:Sized{
	self.children_of_type()
}
fn anyToken(&self, i: usize) -> Option<Rc<AnyTokenContextAll<'input>>> where Self:Sized{
	self.child_of_type(i)
}

}

impl<'input> DirectiveContextAttrs<'input> for DirectiveContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn directive(&mut self,)
	-> Result<Rc<DirectiveContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = DirectiveContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 6, RULE_directive);
        let mut _localctx: Rc<DirectiveContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(59);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Prices_WS {
				{
				recog.base.set_state(58);
				recog.base.match_token(Prices_WS,&mut recog.err_handler)?;

				}
			}

			recog.base.set_state(61);
			recog.base.match_token(Prices_SEMI,&mut recog.err_handler)?;

			recog.base.set_state(66);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 100663294) != 0) {
				{
				recog.base.set_state(64);
				recog.err_handler.sync(&mut recog.base)?;
				match recog.base.input.la(1) {
				Prices_WS 
					=> {
						{
						recog.base.set_state(62);
						recog.base.match_token(Prices_WS,&mut recog.err_handler)?;

						}
					}

				Prices_ACCOUNT |Prices_OPENING |Prices_NAME |Prices_PRICE_DIRECTIVE |
				Prices_DOUBLE_LBRACE |Prices_DOUBLE_RBRACE |Prices_ATAT |Prices_LBRACE |
				Prices_RBRACE |Prices_AT |Prices_SEMI |Prices_COMMA |Prices_COLON |Prices_STAR |
				Prices_BANG |Prices_PLUS |Prices_MINUS |Prices_DOT |Prices_DATETIME |
				Prices_GROUPED_NUMBER |Prices_NUMBER |Prices_QUOTED |Prices_ALNUM |Prices_OTHER 
					=> {
						{
						/*InvokeRule anyToken*/
						recog.base.set_state(63);
						recog.anyToken()?;

						}
					}

					_ => Err(ANTLRError::NoAltError(NoViableAltError::new(&mut recog.base)))?
				}
				}
				recog.base.set_state(68);
				recog.err_handler.sync(&mut recog.base)?;
				_la = recog.base.input.la(1);
			}
			/*InvokeRule lineEnd*/
			recog.base.set_state(69);
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

impl<'input> PricesContext<'input> for BlankLineContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for BlankLineContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for BlankLineContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_blankLine(self);
	}
}

impl<'input> CustomRuleContext<'input> for BlankLineContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_blankLine }
	//fn type_rule_index() -> usize where Self: Sized { RULE_blankLine }
}
antlr4rust::tid!{BlankLineContextExt<'a>}

impl<'input> BlankLineContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<BlankLineContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,BlankLineContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait BlankLineContextAttrs<'input>: PricesContext<'input> + BorrowMut<BlankLineContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token NEWLINE
/// Returns `None` if there is no child corresponding to token NEWLINE
fn NEWLINE(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_NEWLINE, 0)
}
/// Retrieves first TerminalNode corresponding to token WS
/// Returns `None` if there is no child corresponding to token WS
fn WS(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_WS, 0)
}

}

impl<'input> BlankLineContextAttrs<'input> for BlankLineContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn blankLine(&mut self,)
	-> Result<Rc<BlankLineContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = BlankLineContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 8, RULE_blankLine);
        let mut _localctx: Rc<BlankLineContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(72);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Prices_WS {
				{
				recog.base.set_state(71);
				recog.base.match_token(Prices_WS,&mut recog.err_handler)?;

				}
			}

			recog.base.set_state(74);
			recog.base.match_token(Prices_NEWLINE,&mut recog.err_handler)?;

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

impl<'input> PricesContext<'input> for CommodityInputContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for CommodityInputContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for CommodityInputContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_commodityInput(self);
	}
}

impl<'input> CustomRuleContext<'input> for CommodityInputContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_commodityInput }
	//fn type_rule_index() -> usize where Self: Sized { RULE_commodityInput }
}
antlr4rust::tid!{CommodityInputContextExt<'a>}

impl<'input> CommodityInputContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<CommodityInputContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CommodityInputContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CommodityInputContextAttrs<'input>: PricesContext<'input> + BorrowMut<CommodityInputContextExt<'input>>{

fn commodity(&self) -> Option<Rc<CommodityContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_EOF, 0)
}

}

impl<'input> CommodityInputContextAttrs<'input> for CommodityInputContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn commodityInput(&mut self,)
	-> Result<Rc<CommodityInputContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CommodityInputContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 10, RULE_commodityInput);
        let mut _localctx: Rc<CommodityInputContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule commodity*/
			recog.base.set_state(76);
			recog.commodity()?;

			recog.base.set_state(77);
			recog.base.match_token(Prices_EOF,&mut recog.err_handler)?;

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

impl<'input> PricesContext<'input> for AmountInputContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for AmountInputContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for AmountInputContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_amountInput(self);
	}
}

impl<'input> CustomRuleContext<'input> for AmountInputContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_amountInput }
	//fn type_rule_index() -> usize where Self: Sized { RULE_amountInput }
}
antlr4rust::tid!{AmountInputContextExt<'a>}

impl<'input> AmountInputContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<AmountInputContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AmountInputContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AmountInputContextAttrs<'input>: PricesContext<'input> + BorrowMut<AmountInputContextExt<'input>>{

fn amount(&self) -> Option<Rc<AmountContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_EOF, 0)
}

}

impl<'input> AmountInputContextAttrs<'input> for AmountInputContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn amountInput(&mut self,)
	-> Result<Rc<AmountInputContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AmountInputContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 12, RULE_amountInput);
        let mut _localctx: Rc<AmountInputContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule amount*/
			recog.base.set_state(79);
			recog.amount()?;

			recog.base.set_state(80);
			recog.base.match_token(Prices_EOF,&mut recog.err_handler)?;

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

impl<'input> PricesContext<'input> for CommodityContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for CommodityContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for CommodityContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_commodity(self);
	}
}

impl<'input> CustomRuleContext<'input> for CommodityContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_commodity }
	//fn type_rule_index() -> usize where Self: Sized { RULE_commodity }
}
antlr4rust::tid!{CommodityContextExt<'a>}

impl<'input> CommodityContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<CommodityContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CommodityContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CommodityContextAttrs<'input>: PricesContext<'input> + BorrowMut<CommodityContextExt<'input>>{

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

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn commodity(&mut self,)
	-> Result<Rc<CommodityContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CommodityContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 14, RULE_commodity);
        let mut _localctx: Rc<CommodityContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			/*InvokeRule commodityHead*/
			recog.base.set_state(82);
			recog.commodityHead()?;

			recog.base.set_state(86);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			while (((_la) & !0x3f) == 0 && ((1usize << _la) & 10887198) != 0) {
				{
				{
				/*InvokeRule commodityTail*/
				recog.base.set_state(83);
				recog.commodityTail()?;

				}
				}
				recog.base.set_state(88);
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
//------------------- commodityHead ----------------
pub type CommodityHeadContextAll<'input> = CommodityHeadContext<'input>;


pub type CommodityHeadContext<'input> = BaseParserRuleContext<'input,CommodityHeadContextExt<'input>>;

#[derive(Clone)]
pub struct CommodityHeadContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> PricesContext<'input> for CommodityHeadContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for CommodityHeadContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for CommodityHeadContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_commodityHead(self);
	}
}

impl<'input> CustomRuleContext<'input> for CommodityHeadContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_commodityHead }
	//fn type_rule_index() -> usize where Self: Sized { RULE_commodityHead }
}
antlr4rust::tid!{CommodityHeadContextExt<'a>}

impl<'input> CommodityHeadContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<CommodityHeadContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CommodityHeadContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CommodityHeadContextAttrs<'input>: PricesContext<'input> + BorrowMut<CommodityHeadContextExt<'input>>{

fn wordToken(&self) -> Option<Rc<WordTokenContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token NUMBER
/// Returns `None` if there is no child corresponding to token NUMBER
fn NUMBER(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_NUMBER, 0)
}

}

impl<'input> CommodityHeadContextAttrs<'input> for CommodityHeadContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn commodityHead(&mut self,)
	-> Result<Rc<CommodityHeadContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CommodityHeadContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 16, RULE_commodityHead);
        let mut _localctx: Rc<CommodityHeadContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(91);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Prices_ACCOUNT |Prices_OPENING |Prices_NAME |Prices_PRICE_DIRECTIVE |
			Prices_ALNUM 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule wordToken*/
					recog.base.set_state(89);
					recog.wordToken()?;

					}
				}

			Prices_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(90);
					recog.base.match_token(Prices_NUMBER,&mut recog.err_handler)?;

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

impl<'input> PricesContext<'input> for CommodityTailContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for CommodityTailContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for CommodityTailContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_commodityTail(self);
	}
}

impl<'input> CustomRuleContext<'input> for CommodityTailContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_commodityTail }
	//fn type_rule_index() -> usize where Self: Sized { RULE_commodityTail }
}
antlr4rust::tid!{CommodityTailContextExt<'a>}

impl<'input> CommodityTailContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<CommodityTailContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,CommodityTailContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait CommodityTailContextAttrs<'input>: PricesContext<'input> + BorrowMut<CommodityTailContextExt<'input>>{

fn commodityHead(&self) -> Option<Rc<CommodityHeadContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token DOT
/// Returns `None` if there is no child corresponding to token DOT
fn DOT(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_DOT, 0)
}
/// Retrieves first TerminalNode corresponding to token MINUS
/// Returns `None` if there is no child corresponding to token MINUS
fn MINUS(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_MINUS, 0)
}
/// Retrieves first TerminalNode corresponding to token COLON
/// Returns `None` if there is no child corresponding to token COLON
fn COLON(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_COLON, 0)
}

}

impl<'input> CommodityTailContextAttrs<'input> for CommodityTailContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn commodityTail(&mut self,)
	-> Result<Rc<CommodityTailContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = CommodityTailContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 18, RULE_commodityTail);
        let mut _localctx: Rc<CommodityTailContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(97);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Prices_ACCOUNT |Prices_OPENING |Prices_NAME |Prices_PRICE_DIRECTIVE |
			Prices_NUMBER |Prices_ALNUM 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule commodityHead*/
					recog.base.set_state(93);
					recog.commodityHead()?;

					}
				}

			Prices_DOT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(94);
					recog.base.match_token(Prices_DOT,&mut recog.err_handler)?;

					}
				}

			Prices_MINUS 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(95);
					recog.base.match_token(Prices_MINUS,&mut recog.err_handler)?;

					}
				}

			Prices_COLON 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(96);
					recog.base.match_token(Prices_COLON,&mut recog.err_handler)?;

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

impl<'input> PricesContext<'input> for AmountContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for AmountContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for AmountContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_amount(self);
	}
}

impl<'input> CustomRuleContext<'input> for AmountContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_amount }
	//fn type_rule_index() -> usize where Self: Sized { RULE_amount }
}
antlr4rust::tid!{AmountContextExt<'a>}

impl<'input> AmountContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<AmountContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AmountContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AmountContextAttrs<'input>: PricesContext<'input> + BorrowMut<AmountContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token NUMBER
/// Returns `None` if there is no child corresponding to token NUMBER
fn NUMBER(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token GROUPED_NUMBER
/// Returns `None` if there is no child corresponding to token GROUPED_NUMBER
fn GROUPED_NUMBER(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_GROUPED_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token PLUS
/// Returns `None` if there is no child corresponding to token PLUS
fn PLUS(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_PLUS, 0)
}
/// Retrieves first TerminalNode corresponding to token MINUS
/// Returns `None` if there is no child corresponding to token MINUS
fn MINUS(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_MINUS, 0)
}

}

impl<'input> AmountContextAttrs<'input> for AmountContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn amount(&mut self,)
	-> Result<Rc<AmountContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AmountContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 20, RULE_amount);
        let mut _localctx: Rc<AmountContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(100);
			recog.err_handler.sync(&mut recog.base)?;
			_la = recog.base.input.la(1);
			if _la==Prices_PLUS || _la==Prices_MINUS {
				{
				recog.base.set_state(99);
				_la = recog.base.input.la(1);
				if { !(_la==Prices_PLUS || _la==Prices_MINUS) } {
					recog.err_handler.recover_inline(&mut recog.base)?;

				}
				else {
					if  recog.base.input.la(1)==TOKEN_EOF { recog.base.matched_eof = true };
					recog.err_handler.report_match(&mut recog.base);
					recog.base.consume(&mut recog.err_handler);
				}
				}
			}

			recog.base.set_state(102);
			_la = recog.base.input.la(1);
			if { !(_la==Prices_GROUPED_NUMBER || _la==Prices_NUMBER) } {
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
//------------------- wordToken ----------------
pub type WordTokenContextAll<'input> = WordTokenContext<'input>;


pub type WordTokenContext<'input> = BaseParserRuleContext<'input,WordTokenContextExt<'input>>;

#[derive(Clone)]
pub struct WordTokenContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> PricesContext<'input> for WordTokenContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for WordTokenContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for WordTokenContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_wordToken(self);
	}
}

impl<'input> CustomRuleContext<'input> for WordTokenContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_wordToken }
	//fn type_rule_index() -> usize where Self: Sized { RULE_wordToken }
}
antlr4rust::tid!{WordTokenContextExt<'a>}

impl<'input> WordTokenContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<WordTokenContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,WordTokenContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait WordTokenContextAttrs<'input>: PricesContext<'input> + BorrowMut<WordTokenContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token ALNUM
/// Returns `None` if there is no child corresponding to token ALNUM
fn ALNUM(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_ALNUM, 0)
}
/// Retrieves first TerminalNode corresponding to token ACCOUNT
/// Returns `None` if there is no child corresponding to token ACCOUNT
fn ACCOUNT(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_ACCOUNT, 0)
}
/// Retrieves first TerminalNode corresponding to token OPENING
/// Returns `None` if there is no child corresponding to token OPENING
fn OPENING(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_OPENING, 0)
}
/// Retrieves first TerminalNode corresponding to token NAME
/// Returns `None` if there is no child corresponding to token NAME
fn NAME(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_NAME, 0)
}
/// Retrieves first TerminalNode corresponding to token PRICE_DIRECTIVE
/// Returns `None` if there is no child corresponding to token PRICE_DIRECTIVE
fn PRICE_DIRECTIVE(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_PRICE_DIRECTIVE, 0)
}

}

impl<'input> WordTokenContextAttrs<'input> for WordTokenContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn wordToken(&mut self,)
	-> Result<Rc<WordTokenContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = WordTokenContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 22, RULE_wordToken);
        let mut _localctx: Rc<WordTokenContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(104);
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
//------------------- anyToken ----------------
pub type AnyTokenContextAll<'input> = AnyTokenContext<'input>;


pub type AnyTokenContext<'input> = BaseParserRuleContext<'input,AnyTokenContextExt<'input>>;

#[derive(Clone)]
pub struct AnyTokenContextExt<'input>{
ph:PhantomData<&'input str>
}

impl<'input> PricesContext<'input> for AnyTokenContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for AnyTokenContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for AnyTokenContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_anyToken(self);
	}
}

impl<'input> CustomRuleContext<'input> for AnyTokenContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_anyToken }
	//fn type_rule_index() -> usize where Self: Sized { RULE_anyToken }
}
antlr4rust::tid!{AnyTokenContextExt<'a>}

impl<'input> AnyTokenContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<AnyTokenContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,AnyTokenContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait AnyTokenContextAttrs<'input>: PricesContext<'input> + BorrowMut<AnyTokenContextExt<'input>>{

fn wordToken(&self) -> Option<Rc<WordTokenContextAll<'input>>> where Self:Sized{
	self.child_of_type(0)
}
/// Retrieves first TerminalNode corresponding to token NUMBER
/// Returns `None` if there is no child corresponding to token NUMBER
fn NUMBER(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token GROUPED_NUMBER
/// Returns `None` if there is no child corresponding to token GROUPED_NUMBER
fn GROUPED_NUMBER(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_GROUPED_NUMBER, 0)
}
/// Retrieves first TerminalNode corresponding to token DATETIME
/// Returns `None` if there is no child corresponding to token DATETIME
fn DATETIME(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_DATETIME, 0)
}
/// Retrieves first TerminalNode corresponding to token QUOTED
/// Returns `None` if there is no child corresponding to token QUOTED
fn QUOTED(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_QUOTED, 0)
}
/// Retrieves first TerminalNode corresponding to token DOUBLE_LBRACE
/// Returns `None` if there is no child corresponding to token DOUBLE_LBRACE
fn DOUBLE_LBRACE(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_DOUBLE_LBRACE, 0)
}
/// Retrieves first TerminalNode corresponding to token DOUBLE_RBRACE
/// Returns `None` if there is no child corresponding to token DOUBLE_RBRACE
fn DOUBLE_RBRACE(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_DOUBLE_RBRACE, 0)
}
/// Retrieves first TerminalNode corresponding to token ATAT
/// Returns `None` if there is no child corresponding to token ATAT
fn ATAT(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_ATAT, 0)
}
/// Retrieves first TerminalNode corresponding to token LBRACE
/// Returns `None` if there is no child corresponding to token LBRACE
fn LBRACE(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_LBRACE, 0)
}
/// Retrieves first TerminalNode corresponding to token RBRACE
/// Returns `None` if there is no child corresponding to token RBRACE
fn RBRACE(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_RBRACE, 0)
}
/// Retrieves first TerminalNode corresponding to token AT
/// Returns `None` if there is no child corresponding to token AT
fn AT(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_AT, 0)
}
/// Retrieves first TerminalNode corresponding to token SEMI
/// Returns `None` if there is no child corresponding to token SEMI
fn SEMI(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_SEMI, 0)
}
/// Retrieves first TerminalNode corresponding to token COMMA
/// Returns `None` if there is no child corresponding to token COMMA
fn COMMA(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_COMMA, 0)
}
/// Retrieves first TerminalNode corresponding to token COLON
/// Returns `None` if there is no child corresponding to token COLON
fn COLON(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_COLON, 0)
}
/// Retrieves first TerminalNode corresponding to token STAR
/// Returns `None` if there is no child corresponding to token STAR
fn STAR(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_STAR, 0)
}
/// Retrieves first TerminalNode corresponding to token BANG
/// Returns `None` if there is no child corresponding to token BANG
fn BANG(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_BANG, 0)
}
/// Retrieves first TerminalNode corresponding to token PLUS
/// Returns `None` if there is no child corresponding to token PLUS
fn PLUS(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_PLUS, 0)
}
/// Retrieves first TerminalNode corresponding to token MINUS
/// Returns `None` if there is no child corresponding to token MINUS
fn MINUS(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_MINUS, 0)
}
/// Retrieves first TerminalNode corresponding to token DOT
/// Returns `None` if there is no child corresponding to token DOT
fn DOT(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_DOT, 0)
}
/// Retrieves first TerminalNode corresponding to token OTHER
/// Returns `None` if there is no child corresponding to token OTHER
fn OTHER(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_OTHER, 0)
}

}

impl<'input> AnyTokenContextAttrs<'input> for AnyTokenContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn anyToken(&mut self,)
	-> Result<Rc<AnyTokenContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = AnyTokenContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 24, RULE_anyToken);
        let mut _localctx: Rc<AnyTokenContextAll> = _localctx;
		let result: Result<(), ANTLRError> = (|| {

			recog.base.set_state(126);
			recog.err_handler.sync(&mut recog.base)?;
			match recog.base.input.la(1) {
			Prices_ACCOUNT |Prices_OPENING |Prices_NAME |Prices_PRICE_DIRECTIVE |
			Prices_ALNUM 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
					recog.base.enter_outer_alt(None, 1)?;
					{
					/*InvokeRule wordToken*/
					recog.base.set_state(106);
					recog.wordToken()?;

					}
				}

			Prices_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 2)?;
					recog.base.enter_outer_alt(None, 2)?;
					{
					recog.base.set_state(107);
					recog.base.match_token(Prices_NUMBER,&mut recog.err_handler)?;

					}
				}

			Prices_GROUPED_NUMBER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 3)?;
					recog.base.enter_outer_alt(None, 3)?;
					{
					recog.base.set_state(108);
					recog.base.match_token(Prices_GROUPED_NUMBER,&mut recog.err_handler)?;

					}
				}

			Prices_DATETIME 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 4)?;
					recog.base.enter_outer_alt(None, 4)?;
					{
					recog.base.set_state(109);
					recog.base.match_token(Prices_DATETIME,&mut recog.err_handler)?;

					}
				}

			Prices_QUOTED 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 5)?;
					recog.base.enter_outer_alt(None, 5)?;
					{
					recog.base.set_state(110);
					recog.base.match_token(Prices_QUOTED,&mut recog.err_handler)?;

					}
				}

			Prices_DOUBLE_LBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 6)?;
					recog.base.enter_outer_alt(None, 6)?;
					{
					recog.base.set_state(111);
					recog.base.match_token(Prices_DOUBLE_LBRACE,&mut recog.err_handler)?;

					}
				}

			Prices_DOUBLE_RBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 7)?;
					recog.base.enter_outer_alt(None, 7)?;
					{
					recog.base.set_state(112);
					recog.base.match_token(Prices_DOUBLE_RBRACE,&mut recog.err_handler)?;

					}
				}

			Prices_ATAT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 8)?;
					recog.base.enter_outer_alt(None, 8)?;
					{
					recog.base.set_state(113);
					recog.base.match_token(Prices_ATAT,&mut recog.err_handler)?;

					}
				}

			Prices_LBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 9)?;
					recog.base.enter_outer_alt(None, 9)?;
					{
					recog.base.set_state(114);
					recog.base.match_token(Prices_LBRACE,&mut recog.err_handler)?;

					}
				}

			Prices_RBRACE 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 10)?;
					recog.base.enter_outer_alt(None, 10)?;
					{
					recog.base.set_state(115);
					recog.base.match_token(Prices_RBRACE,&mut recog.err_handler)?;

					}
				}

			Prices_AT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 11)?;
					recog.base.enter_outer_alt(None, 11)?;
					{
					recog.base.set_state(116);
					recog.base.match_token(Prices_AT,&mut recog.err_handler)?;

					}
				}

			Prices_SEMI 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 12)?;
					recog.base.enter_outer_alt(None, 12)?;
					{
					recog.base.set_state(117);
					recog.base.match_token(Prices_SEMI,&mut recog.err_handler)?;

					}
				}

			Prices_COMMA 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 13)?;
					recog.base.enter_outer_alt(None, 13)?;
					{
					recog.base.set_state(118);
					recog.base.match_token(Prices_COMMA,&mut recog.err_handler)?;

					}
				}

			Prices_COLON 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 14)?;
					recog.base.enter_outer_alt(None, 14)?;
					{
					recog.base.set_state(119);
					recog.base.match_token(Prices_COLON,&mut recog.err_handler)?;

					}
				}

			Prices_STAR 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 15)?;
					recog.base.enter_outer_alt(None, 15)?;
					{
					recog.base.set_state(120);
					recog.base.match_token(Prices_STAR,&mut recog.err_handler)?;

					}
				}

			Prices_BANG 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 16)?;
					recog.base.enter_outer_alt(None, 16)?;
					{
					recog.base.set_state(121);
					recog.base.match_token(Prices_BANG,&mut recog.err_handler)?;

					}
				}

			Prices_PLUS 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 17)?;
					recog.base.enter_outer_alt(None, 17)?;
					{
					recog.base.set_state(122);
					recog.base.match_token(Prices_PLUS,&mut recog.err_handler)?;

					}
				}

			Prices_MINUS 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 18)?;
					recog.base.enter_outer_alt(None, 18)?;
					{
					recog.base.set_state(123);
					recog.base.match_token(Prices_MINUS,&mut recog.err_handler)?;

					}
				}

			Prices_DOT 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 19)?;
					recog.base.enter_outer_alt(None, 19)?;
					{
					recog.base.set_state(124);
					recog.base.match_token(Prices_DOT,&mut recog.err_handler)?;

					}
				}

			Prices_OTHER 
				=> {
					//recog.base.enter_outer_alt(_localctx.clone(), 20)?;
					recog.base.enter_outer_alt(None, 20)?;
					{
					recog.base.set_state(125);
					recog.base.match_token(Prices_OTHER,&mut recog.err_handler)?;

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

impl<'input> PricesContext<'input> for LineEndContext<'input>{}

impl<'input,'a> Listenable<dyn PricesNoOpListener<'input> + 'a> for LineEndContext<'input>{}

impl<'input,'a> Visitable<dyn PricesVisitor<'input> + 'a> for LineEndContext<'input>{
	fn accept(&self,visitor: &mut (dyn PricesVisitor<'input> + 'a)) {
		visitor.visit_lineEnd(self);
	}
}

impl<'input> CustomRuleContext<'input> for LineEndContextExt<'input>{
	type TF = LocalTokenFactory<'input>;
	type Ctx = PricesContextType;
	fn get_rule_index(&self) -> usize { RULE_lineEnd }
	//fn type_rule_index() -> usize where Self: Sized { RULE_lineEnd }
}
antlr4rust::tid!{LineEndContextExt<'a>}

impl<'input> LineEndContextExt<'input>{
	fn new(parent: Option<Rc<dyn PricesContext<'input> + 'input > >, invoking_state: i32) -> Rc<LineEndContextAll<'input>> {
		Rc::new(
			BaseParserRuleContext::new_parser_ctx(parent, invoking_state,LineEndContextExt{

				ph:PhantomData
			}),
		)
	}
}

pub trait LineEndContextAttrs<'input>: PricesContext<'input> + BorrowMut<LineEndContextExt<'input>>{

/// Retrieves first TerminalNode corresponding to token NEWLINE
/// Returns `None` if there is no child corresponding to token NEWLINE
fn NEWLINE(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_NEWLINE, 0)
}
/// Retrieves first TerminalNode corresponding to token EOF
/// Returns `None` if there is no child corresponding to token EOF
fn EOF(&self) -> Option<Rc<TerminalNode<'input,PricesContextType>>> where Self:Sized{
	self.get_token(Prices_EOF, 0)
}

}

impl<'input> LineEndContextAttrs<'input> for LineEndContext<'input>{}

impl<'input, I> Prices<'input, I>
where
    I: TokenStream<'input, TF = LocalTokenFactory<'input> > + TidAble<'input>,
{
	pub fn lineEnd(&mut self,)
	-> Result<Rc<LineEndContextAll<'input>>,ANTLRError> {
		let mut recog = self;
		let _parentctx = recog.ctx.take();
		let mut _localctx = LineEndContextExt::new(_parentctx.clone(), recog.base.get_state());
        recog.base.enter_rule(_localctx.clone(), 26, RULE_lineEnd);
        let mut _localctx: Rc<LineEndContextAll> = _localctx;
		let mut _la: i32 = -1;
		let result: Result<(), ANTLRError> = (|| {

			//recog.base.enter_outer_alt(_localctx.clone(), 1)?;
			recog.base.enter_outer_alt(None, 1)?;
			{
			recog.base.set_state(128);
			_la = recog.base.input.la(1);
			if { !(_la==Prices_EOF || _la==Prices_NEWLINE) } {
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
		4, 1, 26, 131, 2, 0, 7, 0, 2, 1, 7, 1, 2, 2, 7, 2, 2, 3, 7, 3, 2, 4, 7, 
		4, 2, 5, 7, 5, 2, 6, 7, 6, 2, 7, 7, 7, 2, 8, 7, 8, 2, 9, 7, 9, 2, 10, 
		7, 10, 2, 11, 7, 11, 2, 12, 7, 12, 2, 13, 7, 13, 1, 0, 5, 0, 30, 8, 0, 
		10, 0, 12, 0, 33, 9, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 3, 1, 40, 8, 1, 
		1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 5, 2, 52, 
		8, 2, 10, 2, 12, 2, 55, 9, 2, 1, 2, 1, 2, 1, 3, 3, 3, 60, 8, 3, 1, 3, 
		1, 3, 1, 3, 5, 3, 65, 8, 3, 10, 3, 12, 3, 68, 9, 3, 1, 3, 1, 3, 1, 4, 
		3, 4, 73, 8, 4, 1, 4, 1, 4, 1, 5, 1, 5, 1, 5, 1, 6, 1, 6, 1, 6, 1, 7, 
		1, 7, 5, 7, 85, 8, 7, 10, 7, 12, 7, 88, 9, 7, 1, 8, 1, 8, 3, 8, 92, 8, 
		8, 1, 9, 1, 9, 1, 9, 1, 9, 3, 9, 98, 8, 9, 1, 10, 3, 10, 101, 8, 10, 1, 
		10, 1, 10, 1, 11, 1, 11, 1, 12, 1, 12, 1, 12, 1, 12, 1, 12, 1, 12, 1, 
		12, 1, 12, 1, 12, 1, 12, 1, 12, 1, 12, 1, 12, 1, 12, 1, 12, 1, 12, 1, 
		12, 1, 12, 1, 12, 1, 12, 3, 12, 127, 8, 12, 1, 13, 1, 13, 1, 13, 0, 0, 
		14, 0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 0, 4, 1, 0, 16, 
		17, 1, 0, 20, 21, 2, 0, 1, 4, 23, 23, 1, 1, 25, 25, 149, 0, 31, 1, 0, 
		0, 0, 2, 39, 1, 0, 0, 0, 4, 41, 1, 0, 0, 0, 6, 59, 1, 0, 0, 0, 8, 72, 
		1, 0, 0, 0, 10, 76, 1, 0, 0, 0, 12, 79, 1, 0, 0, 0, 14, 82, 1, 0, 0, 0, 
		16, 91, 1, 0, 0, 0, 18, 97, 1, 0, 0, 0, 20, 100, 1, 0, 0, 0, 22, 104, 
		1, 0, 0, 0, 24, 126, 1, 0, 0, 0, 26, 128, 1, 0, 0, 0, 28, 30, 3, 2, 1, 
		0, 29, 28, 1, 0, 0, 0, 30, 33, 1, 0, 0, 0, 31, 29, 1, 0, 0, 0, 31, 32, 
		1, 0, 0, 0, 32, 34, 1, 0, 0, 0, 33, 31, 1, 0, 0, 0, 34, 35, 5, 0, 0, 1, 
		35, 1, 1, 0, 0, 0, 36, 40, 3, 8, 4, 0, 37, 40, 3, 6, 3, 0, 38, 40, 3, 
		4, 2, 0, 39, 36, 1, 0, 0, 0, 39, 37, 1, 0, 0, 0, 39, 38, 1, 0, 0, 0, 40, 
		3, 1, 0, 0, 0, 41, 42, 5, 4, 0, 0, 42, 43, 5, 24, 0, 0, 43, 44, 5, 19, 
		0, 0, 44, 45, 5, 24, 0, 0, 45, 46, 3, 14, 7, 0, 46, 47, 5, 24, 0, 0, 47, 
		48, 3, 20, 10, 0, 48, 49, 5, 24, 0, 0, 49, 53, 3, 14, 7, 0, 50, 52, 5, 
		24, 0, 0, 51, 50, 1, 0, 0, 0, 52, 55, 1, 0, 0, 0, 53, 51, 1, 0, 0, 0, 
		53, 54, 1, 0, 0, 0, 54, 56, 1, 0, 0, 0, 55, 53, 1, 0, 0, 0, 56, 57, 3, 
		26, 13, 0, 57, 5, 1, 0, 0, 0, 58, 60, 5, 24, 0, 0, 59, 58, 1, 0, 0, 0, 
		59, 60, 1, 0, 0, 0, 60, 61, 1, 0, 0, 0, 61, 66, 5, 11, 0, 0, 62, 65, 5, 
		24, 0, 0, 63, 65, 3, 24, 12, 0, 64, 62, 1, 0, 0, 0, 64, 63, 1, 0, 0, 0, 
		65, 68, 1, 0, 0, 0, 66, 64, 1, 0, 0, 0, 66, 67, 1, 0, 0, 0, 67, 69, 1, 
		0, 0, 0, 68, 66, 1, 0, 0, 0, 69, 70, 3, 26, 13, 0, 70, 7, 1, 0, 0, 0, 
		71, 73, 5, 24, 0, 0, 72, 71, 1, 0, 0, 0, 72, 73, 1, 0, 0, 0, 73, 74, 1, 
		0, 0, 0, 74, 75, 5, 25, 0, 0, 75, 9, 1, 0, 0, 0, 76, 77, 3, 14, 7, 0, 
		77, 78, 5, 0, 0, 1, 78, 11, 1, 0, 0, 0, 79, 80, 3, 20, 10, 0, 80, 81, 
		5, 0, 0, 1, 81, 13, 1, 0, 0, 0, 82, 86, 3, 16, 8, 0, 83, 85, 3, 18, 9, 
		0, 84, 83, 1, 0, 0, 0, 85, 88, 1, 0, 0, 0, 86, 84, 1, 0, 0, 0, 86, 87, 
		1, 0, 0, 0, 87, 15, 1, 0, 0, 0, 88, 86, 1, 0, 0, 0, 89, 92, 3, 22, 11, 
		0, 90, 92, 5, 21, 0, 0, 91, 89, 1, 0, 0, 0, 91, 90, 1, 0, 0, 0, 92, 17, 
		1, 0, 0, 0, 93, 98, 3, 16, 8, 0, 94, 98, 5, 18, 0, 0, 95, 98, 5, 17, 0, 
		0, 96, 98, 5, 13, 0, 0, 97, 93, 1, 0, 0, 0, 97, 94, 1, 0, 0, 0, 97, 95, 
		1, 0, 0, 0, 97, 96, 1, 0, 0, 0, 98, 19, 1, 0, 0, 0, 99, 101, 7, 0, 0, 
		0, 100, 99, 1, 0, 0, 0, 100, 101, 1, 0, 0, 0, 101, 102, 1, 0, 0, 0, 102, 
		103, 7, 1, 0, 0, 103, 21, 1, 0, 0, 0, 104, 105, 7, 2, 0, 0, 105, 23, 1, 
		0, 0, 0, 106, 127, 3, 22, 11, 0, 107, 127, 5, 21, 0, 0, 108, 127, 5, 20, 
		0, 0, 109, 127, 5, 19, 0, 0, 110, 127, 5, 22, 0, 0, 111, 127, 5, 5, 0, 
		0, 112, 127, 5, 6, 0, 0, 113, 127, 5, 7, 0, 0, 114, 127, 5, 8, 0, 0, 115, 
		127, 5, 9, 0, 0, 116, 127, 5, 10, 0, 0, 117, 127, 5, 11, 0, 0, 118, 127, 
		5, 12, 0, 0, 119, 127, 5, 13, 0, 0, 120, 127, 5, 14, 0, 0, 121, 127, 5, 
		15, 0, 0, 122, 127, 5, 16, 0, 0, 123, 127, 5, 17, 0, 0, 124, 127, 5, 18, 
		0, 0, 125, 127, 5, 26, 0, 0, 126, 106, 1, 0, 0, 0, 126, 107, 1, 0, 0, 
		0, 126, 108, 1, 0, 0, 0, 126, 109, 1, 0, 0, 0, 126, 110, 1, 0, 0, 0, 126, 
		111, 1, 0, 0, 0, 126, 112, 1, 0, 0, 0, 126, 113, 1, 0, 0, 0, 126, 114, 
		1, 0, 0, 0, 126, 115, 1, 0, 0, 0, 126, 116, 1, 0, 0, 0, 126, 117, 1, 0, 
		0, 0, 126, 118, 1, 0, 0, 0, 126, 119, 1, 0, 0, 0, 126, 120, 1, 0, 0, 0, 
		126, 121, 1, 0, 0, 0, 126, 122, 1, 0, 0, 0, 126, 123, 1, 0, 0, 0, 126, 
		124, 1, 0, 0, 0, 126, 125, 1, 0, 0, 0, 127, 25, 1, 0, 0, 0, 128, 129, 
		7, 3, 0, 0, 129, 27, 1, 0, 0, 0, 12, 31, 39, 53, 59, 64, 66, 72, 86, 91, 
		97, 100, 126
	];
}
