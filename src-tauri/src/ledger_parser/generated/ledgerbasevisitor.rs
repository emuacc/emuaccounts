
// Generated from Ledger.g4 by ANTLR 4.13.2

use antlr4rust::tree::ParseTreeVisitor;
use super::ledger::*;

// A complete Visitor for a parse tree produced by Ledger.

pub trait LedgerBaseVisitor<'input>:
    ParseTreeVisitor<'input, LedgerContextType> {
	// Visit a parse tree produced by Ledger#file.
	fn visit_file(&mut self, ctx: &FileContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#record.
	fn visit_record(&mut self, ctx: &RecordContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#transaction.
	fn visit_transaction(&mut self, ctx: &TransactionContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#headerLine.
	fn visit_headerline(&mut self, ctx: &HeaderLineContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#status.
	fn visit_status(&mut self, ctx: &StatusContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#headerField.
	fn visit_headerfield(&mut self, ctx: &HeaderFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#postingLine.
	fn visit_postingline(&mut self, ctx: &PostingLineContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#postingComponent.
	fn visit_postingcomponent(&mut self, ctx: &PostingComponentContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#accountDeclaration.
	fn visit_accountdeclaration(&mut self, ctx: &AccountDeclarationContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#accountHeader.
	fn visit_accountheader(&mut self, ctx: &AccountHeaderContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#accountDetailLine.
	fn visit_accountdetailline(&mut self, ctx: &AccountDetailLineContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#openingDetail.
	fn visit_openingdetail(&mut self, ctx: &OpeningDetailContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#nameDetail.
	fn visit_namedetail(&mut self, ctx: &NameDetailContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#nameValue.
	fn visit_namevalue(&mut self, ctx: &NameValueContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#directive.
	fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#blankLine.
	fn visit_blankline(&mut self, ctx: &BlankLineContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#metaComment.
	fn visit_metacomment(&mut self, ctx: &MetaCommentContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#accountInput.
	fn visit_accountinput(&mut self, ctx: &AccountInputContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#commodityInput.
	fn visit_commodityinput(&mut self, ctx: &CommodityInputContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#amountInput.
	fn visit_amountinput(&mut self, ctx: &AmountInputContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#account.
	fn visit_account(&mut self, ctx: &AccountContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#accountSegment.
	fn visit_accountsegment(&mut self, ctx: &AccountSegmentContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#accountPiece.
	fn visit_accountpiece(&mut self, ctx: &AccountPieceContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#commodity.
	fn visit_commodity(&mut self, ctx: &CommodityContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#commodityHead.
	fn visit_commodityhead(&mut self, ctx: &CommodityHeadContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#commodityTail.
	fn visit_commoditytail(&mut self, ctx: &CommodityTailContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#amount.
	fn visit_amount(&mut self, ctx: &AmountContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#lotCostSpec.
	fn visit_lotcostspec(&mut self, ctx: &LotCostSpecContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#costBody.
	fn visit_costbody(&mut self, ctx: &CostBodyContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#costField.
	fn visit_costfield(&mut self, ctx: &CostFieldContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#costValue.
	fn visit_costvalue(&mut self, ctx: &CostValueContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#priceSpec.
	fn visit_pricespec(&mut self, ctx: &PriceSpecContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#bareValue.
	fn visit_barevalue(&mut self, ctx: &BareValueContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#barePiece.
	fn visit_barepiece(&mut self, ctx: &BarePieceContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#wordToken.
	fn visit_wordtoken(&mut self, ctx: &WordTokenContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#looseToken.
	fn visit_loosetoken(&mut self, ctx: &LooseTokenContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#anyToken.
	fn visit_anytoken(&mut self, ctx: &AnyTokenContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Ledger#lineEnd.
	fn visit_lineend(&mut self, ctx: &LineEndContext<'input>) {
            self.visit_children(ctx)
        }

}