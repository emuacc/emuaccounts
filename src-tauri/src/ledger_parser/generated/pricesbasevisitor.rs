
// Generated from Prices.g4 by ANTLR 4.13.2

use antlr4rust::tree::ParseTreeVisitor;
use super::prices::*;

// A complete Visitor for a parse tree produced by Prices.

pub trait PricesBaseVisitor<'input>:
    ParseTreeVisitor<'input, PricesContextType> {
	// Visit a parse tree produced by Prices#pricesFile.
	fn visit_pricesfile(&mut self, ctx: &PricesFileContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#priceRecord.
	fn visit_pricerecord(&mut self, ctx: &PriceRecordContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#priceDirective.
	fn visit_pricedirective(&mut self, ctx: &PriceDirectiveContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#directive.
	fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#blankLine.
	fn visit_blankline(&mut self, ctx: &BlankLineContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#commodityInput.
	fn visit_commodityinput(&mut self, ctx: &CommodityInputContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#amountInput.
	fn visit_amountinput(&mut self, ctx: &AmountInputContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#commodity.
	fn visit_commodity(&mut self, ctx: &CommodityContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#commodityHead.
	fn visit_commodityhead(&mut self, ctx: &CommodityHeadContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#commodityTail.
	fn visit_commoditytail(&mut self, ctx: &CommodityTailContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#amount.
	fn visit_amount(&mut self, ctx: &AmountContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#wordToken.
	fn visit_wordtoken(&mut self, ctx: &WordTokenContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#anyToken.
	fn visit_anytoken(&mut self, ctx: &AnyTokenContext<'input>) {
            self.visit_children(ctx)
        }

	// Visit a parse tree produced by Prices#lineEnd.
	fn visit_lineend(&mut self, ctx: &LineEndContext<'input>) {
            self.visit_children(ctx)
        }

}