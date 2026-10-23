#![allow(nonstandard_style)]
// Generated from Prices.g4 by ANTLR 4.13.2
use antlr4rust::tree::{ParseTreeVisitor,ParseTreeVisitorCompat};
use super::prices::*;

/**
 * This interface defines a complete generic visitor for a parse tree produced
 * by {@link Prices}.
 */
pub trait PricesVisitor<'input>: ParseTreeVisitor<'input,PricesContextType>{
	/**
	 * Visit a parse tree produced by {@link Prices#pricesFile}.
	 * @param ctx the parse tree
	 */
	fn visit_pricesFile(&mut self, ctx: &PricesFileContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#priceRecord}.
	 * @param ctx the parse tree
	 */
	fn visit_priceRecord(&mut self, ctx: &PriceRecordContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#priceDirective}.
	 * @param ctx the parse tree
	 */
	fn visit_priceDirective(&mut self, ctx: &PriceDirectiveContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#directive}.
	 * @param ctx the parse tree
	 */
	fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#blankLine}.
	 * @param ctx the parse tree
	 */
	fn visit_blankLine(&mut self, ctx: &BlankLineContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#commodityInput}.
	 * @param ctx the parse tree
	 */
	fn visit_commodityInput(&mut self, ctx: &CommodityInputContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#amountInput}.
	 * @param ctx the parse tree
	 */
	fn visit_amountInput(&mut self, ctx: &AmountInputContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#commodity}.
	 * @param ctx the parse tree
	 */
	fn visit_commodity(&mut self, ctx: &CommodityContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#commodityHead}.
	 * @param ctx the parse tree
	 */
	fn visit_commodityHead(&mut self, ctx: &CommodityHeadContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#commodityTail}.
	 * @param ctx the parse tree
	 */
	fn visit_commodityTail(&mut self, ctx: &CommodityTailContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#amount}.
	 * @param ctx the parse tree
	 */
	fn visit_amount(&mut self, ctx: &AmountContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#wordToken}.
	 * @param ctx the parse tree
	 */
	fn visit_wordToken(&mut self, ctx: &WordTokenContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#anyToken}.
	 * @param ctx the parse tree
	 */
	fn visit_anyToken(&mut self, ctx: &AnyTokenContext<'input>) { self.visit_children(ctx) }

	/**
	 * Visit a parse tree produced by {@link Prices#lineEnd}.
	 * @param ctx the parse tree
	 */
	fn visit_lineEnd(&mut self, ctx: &LineEndContext<'input>) { self.visit_children(ctx) }

}

pub trait PricesVisitorCompat<'input>:ParseTreeVisitorCompat<'input, Node= PricesContextType>{
	/**
	 * Visit a parse tree produced by {@link Prices#pricesFile}.
	 * @param ctx the parse tree
	 */
		fn visit_pricesFile(&mut self, ctx: &PricesFileContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#priceRecord}.
	 * @param ctx the parse tree
	 */
		fn visit_priceRecord(&mut self, ctx: &PriceRecordContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#priceDirective}.
	 * @param ctx the parse tree
	 */
		fn visit_priceDirective(&mut self, ctx: &PriceDirectiveContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#directive}.
	 * @param ctx the parse tree
	 */
		fn visit_directive(&mut self, ctx: &DirectiveContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#blankLine}.
	 * @param ctx the parse tree
	 */
		fn visit_blankLine(&mut self, ctx: &BlankLineContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#commodityInput}.
	 * @param ctx the parse tree
	 */
		fn visit_commodityInput(&mut self, ctx: &CommodityInputContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#amountInput}.
	 * @param ctx the parse tree
	 */
		fn visit_amountInput(&mut self, ctx: &AmountInputContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#commodity}.
	 * @param ctx the parse tree
	 */
		fn visit_commodity(&mut self, ctx: &CommodityContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#commodityHead}.
	 * @param ctx the parse tree
	 */
		fn visit_commodityHead(&mut self, ctx: &CommodityHeadContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#commodityTail}.
	 * @param ctx the parse tree
	 */
		fn visit_commodityTail(&mut self, ctx: &CommodityTailContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#amount}.
	 * @param ctx the parse tree
	 */
		fn visit_amount(&mut self, ctx: &AmountContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#wordToken}.
	 * @param ctx the parse tree
	 */
		fn visit_wordToken(&mut self, ctx: &WordTokenContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#anyToken}.
	 * @param ctx the parse tree
	 */
		fn visit_anyToken(&mut self, ctx: &AnyTokenContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

	/**
	 * Visit a parse tree produced by {@link Prices#lineEnd}.
	 * @param ctx the parse tree
	 */
		fn visit_lineEnd(&mut self, ctx: &LineEndContext<'input>) -> Self::Return {
			self.visit_children(ctx)
		}

}

impl<'input,T> PricesVisitor<'input> for T
where
	T: PricesVisitorCompat<'input>
{
	fn visit_pricesFile(&mut self, ctx: &PricesFileContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_pricesFile(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_priceRecord(&mut self, ctx: &PriceRecordContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_priceRecord(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_priceDirective(&mut self, ctx: &PriceDirectiveContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_priceDirective(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_directive(&mut self, ctx: &DirectiveContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_directive(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_blankLine(&mut self, ctx: &BlankLineContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_blankLine(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_commodityInput(&mut self, ctx: &CommodityInputContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_commodityInput(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_amountInput(&mut self, ctx: &AmountInputContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_amountInput(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_commodity(&mut self, ctx: &CommodityContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_commodity(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_commodityHead(&mut self, ctx: &CommodityHeadContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_commodityHead(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_commodityTail(&mut self, ctx: &CommodityTailContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_commodityTail(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_amount(&mut self, ctx: &AmountContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_amount(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_wordToken(&mut self, ctx: &WordTokenContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_wordToken(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_anyToken(&mut self, ctx: &AnyTokenContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_anyToken(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

	fn visit_lineEnd(&mut self, ctx: &LineEndContext<'input>){
		let result = <Self as PricesVisitorCompat>::visit_lineEnd(self, ctx);
        *<Self as ParseTreeVisitorCompat>::temp_result(self) = result;
	}

}