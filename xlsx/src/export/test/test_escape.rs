use crate::export::escape::escape_xml;

#[test]
fn test_escape_xml() {
    assert_eq!(escape_xml("all good"), "all good");
    assert_eq!(escape_xml("3 < 4"), "3 &lt; 4");
    assert_eq!(escape_xml("3 > 4"), "3 &gt; 4");
    assert_eq!(escape_xml("3 & 4"), "3 &amp; 4");
    assert_eq!(escape_xml("3 && 4"), "3 &amp;&amp; 4");
    assert_eq!(escape_xml("3 \"literal\" 4"), "3 &quot;literal&quot; 4");
    assert_eq!(
        escape_xml("I don't 'know'"),
        "I don&apos;t &apos;know&apos;"
    );
    assert_eq!(
        escape_xml("This is <>&\"' say"),
        "This is &lt;&gt;&amp;&quot;&apos; say"
    );

    assert_eq!(
        escape_xml("One line\nanother line\n\r"),
        "One line&#xA;another line&#xA;&#xD;"
    );
}
