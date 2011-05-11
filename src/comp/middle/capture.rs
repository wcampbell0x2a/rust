import driver.session;
import front.ast;
import std.Map.hashmap;
import std.Option;
import std.Option.some;
import std.Option.none;
import std.Int;
import std.Vec;
import util.common;

type fn_id_of_local = std.Map.hashmap[ast.def_id, ast.def_id];
type env = rec(mutable vec[ast.def_id] current_context, // fn or obj
               fn_id_of_local idmap,
               session.session sess);

fn current_context(&env e) -> ast.def_id {
    ret e.current_context.(Vec.len(e.current_context) - 1u);
}

fn enter_item(@env e, &@ast.item i) {
    alt (i.node) {
        case (ast.item_fn(?name, _, _, ?id, _)) {
            Vec.push(e.current_context, id);
        }
        case (ast.item_obj(_, _, _, ?ids, _)) {
            Vec.push(e.current_context, ids.ty);
        }
        case (_) {}
    }
}

fn leave_item(@env e, &@ast.item i) {
    alt (i.node) {
        case (ast.item_fn(?name, _, _, ?id, _)) {
            Vec.pop(e.current_context);
        }
        case (ast.item_obj(_, _, _, ?ids, _)) {
            Vec.pop(e.current_context);
        }
        case (_) {}
    }
}

fn walk_expr(@env e, &@ast.expr x) {
    alt (x.node) {
        case (ast.expr_for(?d, _, _, _)) {
            alt (d.node) {
                case (ast.decl_local(?local)) {
                    e.idmap.insert(local.id, current_context(*e));
                }
                case (_) { }
            }
        }
        case (ast.expr_for_each(?d, _, _, _)) {
            alt (d.node) {
                case (ast.decl_local(?local)) {
                    e.idmap.insert(local.id, current_context(*e));
                }
                case (_) { }
            }
        }
        case (ast.expr_path(_, ?def, _)) {
            auto local_id;
            alt (Option.get(def)) {
                case (ast.def_local(?id)) { local_id = id; }
                case (_) { ret; }
            }

            auto df = ast.def_id_of_def(Option.get(def));
            auto def_context = Option.get(e.idmap.find(df));

            if (current_context(*e) != def_context) {
                e.sess.span_err(x.span,
                                "attempted dynamic environment-capture");
            }
        }
        case (_) { }
    }
}

fn walk_block(@env e, &ast.block b) {
    for (@ast.stmt st in b.node.stmts) {
        alt (st.node) {
            case (ast.stmt_decl(?d,_)) {
                alt (d.node) {
                    case (ast.decl_local(?loc)) {
                        e.idmap.insert(loc.id, current_context(*e));
                    }
                    case (_) { }
                }
            }
            case (_) { }
        }
    }
}

fn check_for_captures(session.session sess, @ast.crate crate) {
    let vec[ast.def_id] curctx = vec();
    auto env = @rec(mutable current_context = curctx,
                    idmap = common.new_def_hash[ast.def_id](),
                    sess = sess);
    auto visitor = rec(visit_item_pre = bind enter_item(env, _),
                       visit_item_post = bind leave_item(env, _),
                       visit_block_pre = bind walk_block(env, _),
                       visit_expr_pre = bind walk_expr(env, _)
                       with walk.default_visitor());
    walk.walk_crate(visitor, *crate);
}

// Local Variables:
// mode: rust
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// compile-command: "make -k -C $RBUILD 2>&1 | sed -e 's/\\/x\\//x:\\//g'";
// End:
