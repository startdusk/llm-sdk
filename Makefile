.PHONY: keytest
keytest:
	echo ${OPENAI_API_KEY}
	curl https://api.openai.com/v1/chat/completions \
	  -H "Content-Type: application/json" \
	  -H "Authorization: Bearer ${OPENAI_API_KEY}" \
	  -d '{"model": "gpt-3.5-turbo", "messages": [{"role": "system", "content": "You are a poetic assistant, skilled in explaining complex programming concepts with creative flair."},{"role": "user","content": "Compose a poem that explains the concept of recursion in programming."}]}'

.PHONY: test
test:
	@RUST_LOG=debug cargo nextest run --all-features

.PHONY: fmt
fmt:
	@cargo fmt -- --check
	@cargo clippy --all-targets --all-features --tests --benches -- -D warnings
