(function() {var implementors = {};
implementors["qiniu_http"] = [{"text":"impl&lt;'n&gt; From&lt;&amp;'n str&gt; for HeaderName&lt;'n&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'n&gt; From&lt;&amp;'n str&gt; for HeaderNameOwned","synthetic":false,"types":[]},{"text":"impl&lt;'_&gt; From&lt;String&gt; for HeaderName&lt;'_&gt;","synthetic":false,"types":[]},{"text":"impl From&lt;String&gt; for HeaderNameOwned","synthetic":false,"types":[]},{"text":"impl&lt;'n&gt; From&lt;Cow&lt;'n, str&gt;&gt; for HeaderName&lt;'n&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'n&gt; From&lt;Cow&lt;'n, str&gt;&gt; for HeaderNameOwned","synthetic":false,"types":[]},{"text":"impl&lt;'n&gt; From&lt;HeaderName&lt;'n&gt;&gt; for HeaderNameOwned","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a Method&gt; for Method","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a (dyn Fn(u64, u64) + 'a)&gt; for ProgressCallback&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'_&gt; From&lt;fn(u64, u64)&gt; for ProgressCallback&lt;'_&gt;","synthetic":false,"types":[]}];
implementors["qiniu_ng"] = [{"text":"impl&lt;'_&gt; From&lt;Credential&gt; for Cow&lt;'_, Credential&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a Credential&gt; for Cow&lt;'a, Credential&gt;","synthetic":false,"types":[]},{"text":"impl From&lt;ParseError&gt; for URLParseError","synthetic":false,"types":[]},{"text":"impl From&lt;URLParseError&gt; for ResolveError","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for ResolveError","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for PersistentError","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for PersistentError","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for DomainsError","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for DropBucketError","synthetic":false,"types":[]},{"text":"impl&lt;'_&gt; From&lt;Region&gt; for Cow&lt;'_, Region&gt;","synthetic":false,"types":[]},{"text":"impl From&lt;&amp;'static Region&gt; for Cow&lt;'static, Region&gt;","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for UploadError","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for UploadError","synthetic":false,"types":[]},{"text":"impl From&lt;UploadTokenParseError&gt; for CreateUploaderError","synthetic":false,"types":[]},{"text":"impl From&lt;UploadPolicy&gt; for UploadPolicyBuilder","synthetic":false,"types":[]},{"text":"impl&lt;'p&gt; From&lt;&amp;'p UploadPolicy&gt; for Cow&lt;'p, UploadPolicy&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'_&gt; From&lt;UploadPolicy&gt; for Cow&lt;'_, UploadPolicy&gt;","synthetic":false,"types":[]},{"text":"impl From&lt;Value&gt; for UploadResponse","synthetic":false,"types":[]},{"text":"impl From&lt;Vec&lt;u8&gt;&gt; for UploadResponse","synthetic":false,"types":[]},{"text":"impl&lt;'p&gt; From&lt;Cow&lt;'p, str&gt;&gt; for UploadToken","synthetic":false,"types":[]},{"text":"impl From&lt;String&gt; for UploadToken","synthetic":false,"types":[]},{"text":"impl&lt;'p&gt; From&lt;&amp;'p str&gt; for UploadToken","synthetic":false,"types":[]},{"text":"impl&lt;'p&gt; From&lt;&amp;'p UploadToken&gt; for Cow&lt;'p, UploadToken&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'_&gt; From&lt;UploadToken&gt; for Cow&lt;'_, UploadToken&gt;","synthetic":false,"types":[]},{"text":"impl From&lt;UploadToken&gt; for String","synthetic":false,"types":[]},{"text":"impl From&lt;DecodeError&gt; for UploadTokenParseError","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for UploadTokenParseError","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()