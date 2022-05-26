use super::*;

pub(in super)
fn handle (
    mut trait_: ItemTrait,
) -> Result<TokenStream2>
{
    // Extract the (lifetime) gats.
    #[allow(unstable_name_collisions)]
    let mut lgats: Vec<LGat> =
        trait_
            .items
            .drain_filter(|item| matches!(
                *item, TraitItem::Type(TraitItemType { ref generics , .. })
                if generics.params.is_empty().not()
            ))
            .map(|it| match it {
                | TraitItem::Type(it) => LGat::from_trait_def(it),
                | _ => unreachable!(),
            })
            .collect::<Result<_>>()?
    ;

    let mut ret = quote!();

    // Add the super traits:
    trait_.colon_token.get_or_insert_with(<Token![:]>::default);
    for lgat in lgats {
        let TraitName @ _ = combine_trait_name_and_assoc_type(
            &trait_.ident,
            &lgat.ident,
        );
        let mut generics = trait_.generics.clone();
        for lifetime in lgat.generic_lifetimes.iter().rev() {
            generics.params.insert(0, parse_quote!( #lifetime ));
        }
        generics.params.push({
            let EachImplicitBoundTy =
                lgat.super_types.iter().map(|(lt, SuperTy)| -> Type {
                    // we have a `where SuperTy : 'lt` bound from the
                    // GAT definition; we thus provide an extra and
                    // defaulted (so as to keep it hidden) type:
                    // `&'lt SuperTy`, since be the mere fact of being
                    // *mentioned* it introduces an **implicit** such
                    // bound.
                    //
                    // The bound has to be implicit (vs. the more
                    // straight-forward `where SuperTy : 'lt` approach)
                    // so that the `for<'lt> SuperTrait<'lt>`
                    // quantification correctly holds (the implicit
                    // bound will correctly bound the `for<'lt>` rather
                    // than appear as an unmet requirement outside of a
                    // too-general `for` quantification).
                    parse_quote!(
                        & #lt #SuperTy
                    )
                })
            ;
            parse_quote!(
                __ImplicitBounds = (#(
                    #EachImplicitBoundTy,
                )*)
            )
        });
        let bounds = &lgat.bounds;
        let (intro_generics, where_clause) = (
            &generics.params,
            &generics.where_clause,
        );
        ret.extend(quote!(
            #[allow(warnings, clippy::all)]
            trait #TraitName <#intro_generics>
            #where_clause
            {
                type T : #bounds;
            }
        ));
        //
        let fwd_generics = {
            // Do not include the implicit bound parameter
            drop(generics.params.pop());
            generics.split_for_impl().1
        };
        let each_lgat_lifetime = &lgat.generic_lifetimes;
        trait_.supertraits.push(parse_quote!(
            for<#(#each_lgat_lifetime),*> #TraitName #fwd_generics
        ));
    }

    // time to Conr-"adjugate"
    let trait_ = fold::Fold::fold_item_trait(
        &mut {
            let Trait = &trait_.ident;
            let fwd_generics = trait_.generics.split_for_impl().1;
            ReplaceSelfAssocLtWithSelfAsTraitAssocLt(parse_quote!(
                #Trait #fwd_generics
            ))
        },
        trait_,
    );
    ret.extend(adjugate::adjugate(
        parse::Nothing,
        Item::Trait(trait_),
    ));

    Ok(ret)
}

impl LGat {
    fn from_trait_def (assoc_ty: TraitItemType)
      -> Result<LGat>
    {
        let TraitItemType { attrs, ident, bounds, generics, .. } = assoc_ty;
        let (generic_lifetimes, super_types) = Self::parse_generics(generics)?;
        if let Some((eq, _)) = assoc_ty.default {
            bail! {
                "default GATs are not supported" => eq,
            }
        }
        Ok(LGat {
            attrs,
            ident,
            bounds,
            generic_lifetimes,
            super_types,
            value: None,
        })
    }
}
