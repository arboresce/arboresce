# Independent locator value fixtures

These are original public synthetic fixtures under MIT OR Apache-2.0. The
[observation contract](../../../../../docs/requirements/observation-contract.md)
owns UTF-8 offsets, upright pixel edges, orientation and audio interval meanings.
[cases.json](cases.json) pins that contract, the common contract and unchanged
[primitive semantics](../primitives-v1/semantics.json). All original 11 UTF-8,
nine page and ten audio cases retain their meanings. Supplementary answers are
literals frozen before their consumer; four-corner orientation mappings follow
the owning table, including mirrors and edges rather than pixel indices.

The constructors check explicitly supplied bytes and synthetic numeric metadata.
UTF-8 views borrow exactly their input and never repair, normalize or log it.
Already-upright page regions do not undergo another orientation transform; raw
regions transform all four edges once into the declared upright frame. Audio
spans retain integer milliseconds, channel and actual-duration bounds.

Large synthetic metadata demonstrates arithmetic and wire capacity only. It
does not admit a page count, oversized image, audio codec or original beyond a
qualified processing profile. These APIs do not parse complete locator wire
objects or unsigned JSON tokens, establish representation/render digests, qualify
decoding or sample conversion, grant authority or supply product admission.
All complete wire, source/JCS/budget, representation and real-profile gates remain
required at their consumers. No expected-value helper or generator is imported.

The additive raw-frame discriminators separate physical raw bounds from stored
upright wire capacities. Six mirrored or rotated large raw origins yield literal
small upright rectangles; identity and transpose retain output-capacity rejection.
Additional negative, excessive, outside-frame and page-index inputs fail before
conversion. These synthetic numeric contexts do not qualify oversized media.
