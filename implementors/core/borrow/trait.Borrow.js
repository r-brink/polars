(function() {var implementors = {};
implementors["arrayvec"] = [{"text":"impl&lt;A&gt; Borrow&lt;str&gt; for ArrayString&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Array&lt;Item = u8&gt; + Copy,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;A:&nbsp;Array&gt; Borrow&lt;[&lt;A as Array&gt;::Item]&gt; for ArrayVec&lt;A&gt;","synthetic":false,"types":[]}];
implementors["bstr"] = [{"text":"impl Borrow&lt;BStr&gt; for BString","synthetic":false,"types":[]}];
implementors["crossbeam_epoch"] = [{"text":"impl&lt;T:&nbsp;?Sized + Pointable&gt; Borrow&lt;T&gt; for Owned&lt;T&gt;","synthetic":false,"types":[]}];
implementors["smallvec"] = [{"text":"impl&lt;A:&nbsp;Array&gt; Borrow&lt;[&lt;A as Array&gt;::Item]&gt; for SmallVec&lt;A&gt;","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()