test!(typedef_diff_base_equal, "");
test!(typedef_diff_base, "- type ", "typedef_diff_base", " = char\n", "+ type ", "typedef_diff_base", " = int\n", "- \tsize: 1\n", "+ \tsize: 4\n", "\n");
test!(typedef_diff_anon_equal, "");
test!(typedef_diff_anon, "  type ", "typedef_diff_anon", " = struct <anon>\n", "- \tsize: 1\n", "+ \tsize: 4\n", "  \tmembers:\n", "- \t\t0[1]\tc: char\n", "+ \t\t0[4]\ti: int\n", "\n");
test!(typedef_diff_anon_base, "- type ", "typedef_diff_anon_base", " = char\n", "+ type ", "typedef_diff_anon_base", " = struct <anon>\n", "  \tsize: 1\n", "  \tmembers:\n", "+ \t\t0[1]\tc: char\n", "\n");
test!(typedef_diff_anon_struct_union, "- type ", "typedef_diff_anon_struct_union", " = struct <anon>\n", "+ type ", "typedef_diff_anon_struct_union", " = union <anon>\n", "  \tsize: 1\n", "  \tmembers:\n", "- \t\t0[1]\tc: char\n", "+ \t\t0[1]\tc: char\n", "\n");
test!(typedef_diff_base_anon, "- type ", "typedef_diff_base_anon", " = struct <anon>\n", "+ type ", "typedef_diff_base_anon", " = char\n", "  \tsize: 1\n", "  \tmembers:\n", "- \t\t0[1]\tc: char\n", "\n");
test!(typedef_diff_struct_name, "- type ", "typedef_diff_struct_name", " = struct ", "typedef_diff_struct_name_s1", "\n", "+ type ", "typedef_diff_struct_name", " = struct ", "typedef_diff_struct_name_s2", "\n", "  \tsize: 1\n", "\n");
test!(typedef_diff_struct_size, "  type ", "typedef_diff_struct_size", " = struct s\n", "- \tsize: 1\n", "+ \tsize: 4\n", "\n");
test!(struct_diff_defn_equal, "");
test!(struct_diff_decl_equal, "");
test!(struct_diff_defn_decl, "  struct ", "struct_diff_defn_decl", "\n", "+ \tdeclaration: yes\n", "- \tsize: 1\n", "  \tmembers:\n", "- \t\t0[1]\tc: char\n", "\n");
test!(struct_diff_decl_defn, "  struct ", "struct_diff_decl_defn", "\n", "- \tdeclaration: yes\n", "+ \tsize: 1\n", "  \tmembers:\n", "+ \t\t0[1]\tc: char\n", "\n");
test!(struct_diff_size_equal, "  struct ", "struct_diff_size_equal", "\n", "  \tsize: 2\n", "  \tmembers:\n", "- \t\t0[2]\tc: [char; 2]\n", "+ \t\t0[1]\tc1: char\n", "+ \t\t1[1]\tc2: char\n", "\n");
test!(struct_diff_member, "  struct ", "struct_diff_member", "\n", "- \tsize: 1\n", "+ \tsize: 4\n", "  \tmembers:\n", "- \t\t0[1]\ta: char\n", "+ \t\t0[4]\ta: int\n", "\n");
test!(struct_diff_member_reorder, "  struct ", "struct_diff_member_reorder", "\n", "  \tsize: 7\n", "  \tmembers:\n", "+ \t\t0[2]\td: [char; 2]\n", "+ \t\t2[1]\tc: char\n", "- \t\t0[1]\ta: char\n", "+ \t\t3[1]\ta: char\n", "- \t\t1[1]\tb: char\n", "+ \t\t4[1]\tb: char\n", "- \t\t2[1]\tc: char\n", "- \t\t3[2]\td: [char; 2]\n", "  \t\t5[1]\tx: char\n", "- \t\t6[1]\ty: char\n", "+ \t\t6[1]\tz: char\n", "\n");
test!(struct_diff_recursive_equal, "");
test!(union_diff_defn_equal, "");
test!(union_diff_decl_equal, "");
test!(union_diff_defn_decl, "  union ", "union_diff_defn_decl", "\n", "+ \tdeclaration: yes\n", "- \tsize: 1\n", "  \tmembers:\n", "- \t\t0[1]\tc: char\n", "\n");
test!(union_diff_decl_defn, "  union ", "union_diff_decl_defn", "\n", "- \tdeclaration: yes\n", "+ \tsize: 1\n", "  \tmembers:\n", "+ \t\t0[1]\tc: char\n", "\n");
test!(union_diff_size_equal, "  union ", "union_diff_size_equal", "\n", "  \tsize: 2\n", "  \tmembers:\n", "  \t\t0[2]\t<anon>: struct <anon>\n", "- \t\t\t0[2]\tc: [char; 2]\n", "+ \t\t\t0[1]\tc1: char\n", "+ \t\t\t1[1]\tc2: char\n", "\n");
test!(union_diff_member, "  union ", "union_diff_member", "\n", "- \tsize: 1\n", "+ \tsize: 4\n", "  \tmembers:\n", "- \t\t0[1]\ta: char\n", "+ \t\t0[4]\ta: int\n", "\n");
test!(union_diff_member_reorder, "  union ", "union_diff_member_reorder", "\n", "- \tsize: 1\n", "+ \tsize: 2\n", "  \tmembers:\n", "+ \t\t0[2]\tb: [char; 2]\n", "  \t\t0[1]\ta: char\n", "- \t\t0[1]\tb: char\n", "  \t\t0[1]\tc: char\n", "\n");
test!(member_diff_padding_equal, "");
test!(member_diff_padding, "  struct ", "member_diff_padding", "\n", "  \tsize: 8\n", "  \tmembers:\n", "- \t\t0[1]\ta: [char; 1]\n", "+ \t\t0[2]\ta: [char; 2]\n", "- \t\t1[3]\t<padding>\n", "+ \t\t2[2]\t<padding>\n", "  \t\t4[4]\tb: int\n", "\n");
test!(member_diff_padding_none, "  struct ", "member_diff_padding_none", "\n", "  \tsize: 8\n", "  \tmembers:\n", "- \t\t0[1]\ta: [char; 1]\n", "+ \t\t0[4]\ta: [char; 4]\n", "- \t\t1[3]\t<padding>\n", "  \t\t4[4]\tb: int\n", "\n");
test!(member_diff_bitfield_equal, "  struct ", "member_diff_bitfield_equal", "\n", "  \tsize: 2\n", "  \tmembers:\n", "- \t\t0[1]\ta: char\n", "+ \t\t0[1]\tb: char\n", "  \t\t1[0.1]\tc: char\n", "  \t\t1.1[0.7]\t<padding>\n", "\n");
test!(member_diff_bitfield, "  struct ", "member_diff_bitfield", "\n", "  \tsize: 1\n", "  \tmembers:\n", "- \t\t0[0.1]\ta: char\n", "+ \t\t0[0.2]\ta: char\n", "- \t\t0.1[0.7]\t<padding>\n", "+ \t\t0.2[0.6]\t<padding>\n", "\n");
test!(member_diff_unsized, "  struct ", "member_diff_unsized", "\n", "- \tsize: 2\n", "+ \tsize: 1\n", "  \tmembers:\n", "  \t\t0[1]\ta: char\n", "- \t\t1[1]\tb: [char; 1]\n", "+ \t\t1[??]\tb: [char; ??]\n", "\n");
test!(member_diff_inline_struct_struct_equal, "");
test!(member_diff_inline_struct_struct, "  struct ", "member_diff_inline_struct_struct", "\n", "  \tsize: 1\n", "  \tmembers:\n", "  \t\t0[1]\ta: struct <anon>\n", "- \t\t\t0[1]\tb: char\n", "+ \t\t\t0[1]\tc: char\n", "\n");
test!(member_diff_inline_union_union_equal, "");
test!(member_diff_inline_union_union, "  struct ", "member_diff_inline_union_union", "\n", "  \tsize: 1\n", "  \tmembers:\n", "  \t\t0[1]\ta: union <anon>\n", "- \t\t\t0[1]\tb: char\n", "+ \t\t\t0[1]\tc: char\n", "\n");
test!(member_diff_inline_union_struct, "  struct ", "member_diff_inline_union_struct", "\n", "  \tsize: 1\n", "  \tmembers:\n", "- \t\t0[1]\ta: struct <anon>\n", "+ \t\t0[1]\ta: union <anon>\n", "- \t\t\t0[1]\tb: char\n", "+ \t\t\t0[1]\tb: char\n", "\n");
test!(member_diff_inline_struct_none, "  struct ", "member_diff_inline_struct_none", "\n", "  \tsize: 1\n", "  \tmembers:\n", "- \t\t0[1]\ta: struct <anon>\n", "+ \t\t0[1]\ta: char\n", "- \t\t\t0[1]\tb: char\n", "\n");
test!(member_diff_inline_none_struct, "  struct ", "member_diff_inline_none_struct", "\n", "  \tsize: 1\n", "  \tmembers:\n", "- \t\t0[1]\ta: char\n", "+ \t\t0[1]\ta: struct <anon>\n", "+ \t\t\t0[1]\tb: char\n", "\n");
test!(enum_diff_equal, "");
test!(enum_diff, "  enum ", "enum_diff", "\n", "  \tsize: 4\n", "  \tenumerators:\n", "  \t\tA2(1)\n", "- \t\tB2(2)\n", "+ \t\tC2(2)\n", "- \t\tC2(3)\n", "+ \t\tB2(3)\n", "- \t\tD2(4)\n", "  \t\tE2(5)\n", "+ \t\tF2(6)\n", "\n");
test!(array_diff_equal, "");
test!(array_diff_type, "- type ", "array_diff_type", " = char\n", "+ type ", "array_diff_type", " = C\n", "  \tsize: 1\n", "\n");
test!(array_diff_size, "- type ", "array_diff_size", " = [char; 1]\n", "+ type ", "array_diff_size", " = [char; 2]\n", "- \tsize: 1\n", "+ \tsize: 2\n", "\n");
test!(function_equal, "");
test!(function_diff_return_type, "  fn ", "function_diff_return_type", "\n", "[..]\n", "  \treturn type:\n", "- \t\t[1]\tchar\n", "+ \t\t[4]\tint\n", "\n");
test!(function_diff_variables, "  fn ", "function_diff_variables", "\n", "[..]\n", "  \tvariables:\n", "- \t\t[1]\ta: char\n", "+ \t\t[1]\tb: char\n", "  \t\t[1]\tc: char\n", "- \t\t[1]\td: char\n", "+ \t\t[4]\td: int\n", "- \t\t[1]\te: char\n", "+ \t\t[1]\tf: char\n", "- \t\t[4]\textra: int\n", "  \t\t[1]\tg: char\n", "\n");
test!(variable_equal, "");
test!(variable_diff_size, "- var ", "variable_diff_size", ": [char; 1]\n", "+ var ", "variable_diff_size", ": [char; 2]\n", "[..]", "- \tsize: 1\n", "+ \tsize: 2\n", "\n");
test!(variable_diff_decl, "  var ", "variable_diff_decl", ": int\n", "[..]", "  \tsize: 4\n", "+ \tdeclaration: yes\n", "\n");
