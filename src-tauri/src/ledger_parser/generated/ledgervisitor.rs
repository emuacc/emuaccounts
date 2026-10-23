#![allow(nonstandard_style)]
// Generated from Ledger.g4 by ANTLR 4.13.2
use antlr4rust::tree::{ParseTreeVisitor,ParseTreeVisitorCompat};
use super::ledger::*;

/**
 * This interface defines a complete generic visitor for a parse tree produced
 * by {@link Ledger}.
 */
pub trait LedgerVisitor<'input>: ParseTreeVisitor<'input,LedgerContextType>{
	/**
	 * Visit a parse tree produced by {@link Ledger#file}.
	 * @param ctx the parse tree
	 */
	fn visit_file(&mut self, ctx: &FileContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#record}.
	 * @param ctx the parse tree
	 */
	fn visit_record(&mut self, ctx: &RecordContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#transaction}.
	 * @param ctx the parse tree
	 */
	fn visit_transaction(&mut self, ctx: &TransactionContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#headerLine}.
	 * @param ctx the parse tree
	 */
	fn visit_headerLine(&mut self, ctx: &HeaderLineContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#status}.
	 * @param ctx the parse tree
	 */
	fn visit_status(&mut self, ctx: &StatusContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#headerField}.
	 * @param ctx the parse tree
	 */
	fn visit_headerField(&mut self, ctx: &HeaderFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#postingLine}.
	 * @param ctx the parse tree
	 */
	fn visit_postingLine(&mut self, ctx: &PostingLineContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#postingComponent}.
	 * @param ctx the parse tree
	 */
	fn visit_postingComponent(&mut self, ctx: &PostingComponentContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#accountDeclaration}.
	 * @param ctx the parse tree
	 */
	fn visit_accountDeclaration(&mut self, ctx: &AccountDeclarationContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#accountHeader}.
	 * @param ctx the parse tree
	 */
	fn visit_accountHeader(&mut self, ctx: &AccountHeaderContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#accountDetailLine}.
	 * @param ctx the parse tree
	 */
	fn visit_accountDetailLine(&mut self, ctx: &AccountDetailLineContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#openingDetail}.
	 * @param ctx the parse tree
	 */
	fn visit_openingDetail(&mut self, ctx: &OpeningDetailContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#nameDetail}.
	 * @param ctx the parse tree
	 */
	fn visit_nameDetail(&mut self, ctx: &NameDetailContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#nameValue}.
	 * @param ctx the parse tree
	 */
	fn visit_nameValue(&mut self, ctx: &NameValueContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#directive}.
	 * @param ctx the parse tree
	 */
	fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#blankLine}.
	 * @param ctx the parse tree
	 */
	fn visit_blankLine(&mut self, ctx: &BlankLineContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#metaComment}.
	 * @param ctx the parse tree
	 */
	fn visit_metaComment(&mut self, ctx: &MetaCommentContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#accountInput}.
	 * @param ctx the parse tree
	 */
	fn visit_accountInput(&mut self, ctx: &AccountInputContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#commodityInput}.
	 * @param ctx the parse tree
	 */
	fn visit_commodityInput(&mut self, ctx: &CommodityInputContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#amountInput}.
	 * @param ctx the parse tree
	 */
	fn visit_amountInput(&mut self, ctx: &AmountInputContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#account}.
	 * @param ctx the parse tree
	 */
	fn visit_account(&mut self, ctx: &AccountContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#accountSegment}.
	 * @param ctx the parse tree
	 */
	fn visit_accountSegment(&mut self, ctx: &AccountSegmentContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#accountPiece}.
	 * @param ctx the parse tree
	 */
	fn visit_accountPiece(&mut self, ctx: &AccountPieceContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#commodity}.
	 * @param ctx the parse tree
	 */
	fn visit_commodity(&mut self, ctx: &CommodityContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#commodityHead}.
	 * @param ctx the parse tree
	 */
	fn visit_commodityHead(&mut self, ctx: &CommodityHeadContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#commodityTail}.
	 * @param ctx the parse tree
	 */
	fn visit_commodityTail(&mut self, ctx: &CommodityTailContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#amount}.
	 * @param ctx the parse tree
	 */
	fn visit_amount(&mut self, ctx: &AmountContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#lotCostSpec}.
	 * @param ctx the parse tree
	 */
	fn visit_lotCostSpec(&mut self, ctx: &LotCostSpecContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#costBody}.
	 * @param ctx the parse tree
	 */
	fn visit_costBody(&mut self, ctx: &CostBodyContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#costField}.
	 * @param ctx the parse tree
	 */
	fn visit_costField(&mut self, ctx: &CostFieldContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#costValue}.
	 * @param ctx the parse tree
	 */
	fn visit_costValue(&mut self, ctx: &CostValueContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#priceSpec}.
	 * @param ctx the parse tree
	 */
	fn visit_priceSpec(&mut self, ctx: &PriceSpecContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#bareValue}.
	 * @param ctx the parse tree
	 */
	fn visit_bareValue(&mut self, ctx: &BareValueContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#barePiece}.
	 * @param ctx the parse tree
	 */
	fn visit_barePiece(&mut self, ctx: &BarePieceContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#wordToken}.
	 * @param ctx the parse tree
	 */
	fn visit_wordToken(&mut self, ctx: &WordTokenContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#looseToken}.
	 * @param ctx the parse tree
	 */
	fn visit_looseToken(&mut self, ctx: &LooseTokenContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#anyToken}.
	 * @param ctx the parse tree
	 */
	fn visit_anyToken(&mut self, ctx: &AnyTokenContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Ledger#lineEnd}.
	 * @param ctx the parse tree
	 */
	fn visit_lineEnd(&mut self, ctx: &LineEndContext<'input>) { self.visit_children(ctx) }

}

pub trait LedgerVisitorCompat<'input>:ParseTreeVisitorCompat<'input, Node= LedgerContextType>{
	/**
	 * Visit a parse tree produced by {@link Ledger#file}.
	 * @param ctx the parse tree
	 */
		fn visit_file(&mut self, ctx: &FileContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#record}.
	 * @param ctx the parse tree
	 */
		fn visit_record(&mut self, ctx: &RecordContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#transaction}.
	 * @param ctx the parse tree
	 */
		fn visit_transaction(&mut self, ctx: &TransactionContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#headerLine}.
	 * @param ctx the parse tree
	 */
		fn visit_headerLine(&mut self, ctx: &HeaderLineContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#status}.
	 * @param ctx the parse tree
	 */
		fn visit_status(&mut self, ctx: &StatusContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#headerField}.
	 * @param ctx the parse tree
	 */
		fn visit_headerField(&mut self, ctx: &HeaderFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#postingLine}.
	 * @param ctx the parse tree
	 */
		fn visit_postingLine(&mut self, ctx: &PostingLineContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#postingComponent}.
	 * @param ctx the parse tree
	 */
		fn visit_postingComponent(&mut self, ctx: &PostingComponentContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#accountDeclaration}.
	 * @param ctx the parse tree
	 */
		fn visit_accountDeclaration(&mut self, ctx: &AccountDeclarationContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#accountHeader}.
	 * @param ctx the parse tree
	 */
		fn visit_accountHeader(&mut self, ctx: &AccountHeaderContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#accountDetailLine}.
	 * @param ctx the parse tree
	 */
		fn visit_accountDetailLine(&mut self, ctx: &AccountDetailLineContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#openingDetail}.
	 * @param ctx the parse tree
	 */
		fn visit_openingDetail(&mut self, ctx: &OpeningDetailContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#nameDetail}.
	 * @param ctx the parse tree
	 */
		fn visit_nameDetail(&mut self, ctx: &NameDetailContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#nameValue}.
	 * @param ctx the parse tree
	 */
		fn visit_nameValue(&mut self, ctx: &NameValueContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#directive}.
	 * @param ctx the parse tree
	 */
		fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#blankLine}.
	 * @param ctx the parse tree
	 */
		fn visit_blankLine(&mut self, ctx: &BlankLineContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#metaComment}.
	 * @param ctx the parse tree
	 */
		fn visit_metaComment(&mut self, ctx: &MetaCommentContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#accountInput}.
	 * @param ctx the parse tree
	 */
		fn visit_accountInput(&mut self, ctx: &AccountInputContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#commodityInput}.
	 * @param ctx the parse tree
	 */
		fn visit_commodityInput(&mut self, ctx: &CommodityInputContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#amountInput}.
	 * @param ctx the parse tree
	 */
		fn visit_amountInput(&mut self, ctx: &AmountInputContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#account}.
	 * @param ctx the parse tree
	 */
		fn visit_account(&mut self, ctx: &AccountContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#accountSegment}.
	 * @param ctx the parse tree
	 */
		fn visit_accountSegment(&mut self, ctx: &AccountSegmentContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#accountPiece}.
	 * @param ctx the parse tree
	 */
		fn visit_accountPiece(&mut self, ctx: &AccountPieceContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#commodity}.
	 * @param ctx the parse tree
	 */
		fn visit_commodity(&mut self, ctx: &CommodityContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#commodityHead}.
	 * @param ctx the parse tree
	 */
		fn visit_commodityHead(&mut self, ctx: &CommodityHeadContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#commodityTail}.
	 * @param ctx the parse tree
	 */
		fn visit_commodityTail(&mut self, ctx: &CommodityTailContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#amount}.
	 * @param ctx the parse tree
	 */
		fn visit_amount(&mut self, ctx: &AmountContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#lotCostSpec}.
	 * @param ctx the parse tree
	 */
		fn visit_lotCostSpec(&mut self, ctx: &LotCostSpecContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#costBody}.
	 * @param ctx the parse tree
	 */
		fn visit_costBody(&mut self, ctx: &CostBodyContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#costField}.
	 * @param ctx the parse tree
	 */
		fn visit_costField(&mut self, ctx: &CostFieldContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#costValue}.
	 * @param ctx the parse tree
	 */
		fn visit_costValue(&mut self, ctx: &CostValueContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#priceSpec}.
	 * @param ctx the parse tree
	 */
		fn visit_priceSpec(&mut self, ctx: &PriceSpecContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#bareValue}.
	 * @param ctx the parse tree
	 */
		fn visit_bareValue(&mut self, ctx: &BareValueContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#barePiece}.
	 * @param ctx the parse tree
	 */
		fn visit_barePiece(&mut self, ctx: &BarePieceContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#wordToken}.
	 * @param ctx the parse tree
	 */
		fn visit_wordToken(&mut self, ctx: &WordTokenContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#looseToken}.
	 * @param ctx the parse tree
	 */
		fn visit_looseToken(&mut self, ctx: &LooseTokenContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#anyToken}.
	 * @param ctx the parse tree
	 */
		fn visit_anyToken(&mut self, ctx: &AnyTokenContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Ledger#lineEnd}.
	 * @param ctx the parse tree
	 */
		fn visit_lineEnd(&mut self, ctx: &LineEndContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

}

impl<'input,T> LedgerVisitor<'input> for T
where
	T: LedgerVisitorCompat<'input>
{
	fn visit_file(&mut self, ctx: &FileContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_file(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_record(&mut self, ctx: &RecordContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_record(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_transaction(&mut self, ctx: &TransactionContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_transaction(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_headerLine(&mut self, ctx: &HeaderLineContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_headerLine(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_status(&mut self, ctx: &StatusContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_status(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_headerField(&mut self, ctx: &HeaderFieldContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_headerField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_postingLine(&mut self, ctx: &PostingLineContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_postingLine(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_postingComponent(&mut self, ctx: &PostingComponentContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_postingComponent(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_accountDeclaration(&mut self, ctx: &AccountDeclarationContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_accountDeclaration(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_accountHeader(&mut self, ctx: &AccountHeaderContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_accountHeader(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_accountDetailLine(&mut self, ctx: &AccountDetailLineContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_accountDetailLine(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_openingDetail(&mut self, ctx: &OpeningDetailContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_openingDetail(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_nameDetail(&mut self, ctx: &NameDetailContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_nameDetail(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_nameValue(&mut self, ctx: &NameValueContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_nameValue(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_directive(&mut self, ctx: &DirectiveContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_directive(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_blankLine(&mut self, ctx: &BlankLineContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_blankLine(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_metaComment(&mut self, ctx: &MetaCommentContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_metaComment(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_accountInput(&mut self, ctx: &AccountInputContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_accountInput(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_commodityInput(&mut self, ctx: &CommodityInputContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_commodityInput(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_amountInput(&mut self, ctx: &AmountInputContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_amountInput(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_account(&mut self, ctx: &AccountContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_account(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_accountSegment(&mut self, ctx: &AccountSegmentContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_accountSegment(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_accountPiece(&mut self, ctx: &AccountPieceContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_accountPiece(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_commodity(&mut self, ctx: &CommodityContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_commodity(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_commodityHead(&mut self, ctx: &CommodityHeadContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_commodityHead(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_commodityTail(&mut self, ctx: &CommodityTailContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_commodityTail(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_amount(&mut self, ctx: &AmountContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_amount(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_lotCostSpec(&mut self, ctx: &LotCostSpecContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_lotCostSpec(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_costBody(&mut self, ctx: &CostBodyContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_costBody(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_costField(&mut self, ctx: &CostFieldContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_costField(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_costValue(&mut self, ctx: &CostValueContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_costValue(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_priceSpec(&mut self, ctx: &PriceSpecContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_priceSpec(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_bareValue(&mut self, ctx: &BareValueContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_bareValue(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_barePiece(&mut self, ctx: &BarePieceContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_barePiece(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_wordToken(&mut self, ctx: &WordTokenContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_wordToken(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_looseToken(&mut self, ctx: &LooseTokenContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_looseToken(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_anyToken(&mut self, ctx: &AnyTokenContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_anyToken(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_lineEnd(&mut self, ctx: &LineEndContext<'input>){
		let result = <Self as LedgerVisitorCompat>::visit_lineEnd(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

}