
import java.io.File;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;

import org.apache.commons.io.FileUtils;
import org.apache.commons.lang.StringUtils;

public class BeepIPaConverter {
    public static void main(String args[]) throws Exception {
        {
            Map<String, String> ipamap = new LinkedHashMap<>();
            ipamap.put("ng", "ŋ");
            ipamap.put("ch", "tʃ");
            ipamap.put("jh", "dʒ");
            ipamap.put("th", "θ");
            ipamap.put("dh", "ð");
            ipamap.put("sh", "ʃ");
            ipamap.put("zh", "ʒ");
            ipamap.put("iy", "iː");
            ipamap.put("aa", "ɑː");
            ipamap.put("ao", "ɔː");
            ipamap.put("uw", "uː");
            ipamap.put("er", "ɜː");
            ipamap.put("ih", "ɪ");
            ipamap.put("eh", "ɛ");
            ipamap.put("ae", "æ");
            ipamap.put("ah", "ʌ");
            ipamap.put("oh", "ɒ");
            ipamap.put("uh", "ʊ");
            ipamap.put("ax", "ə");
            ipamap.put("ey", "ɛɪ");
            ipamap.put("ay", "aɪ");
            ipamap.put("oy", "ɔɪ");
            ipamap.put("ow", "oʊ");
            ipamap.put("aw", "aʊ");
            ipamap.put("ia", "ɪə");
            ipamap.put("ea", "ɛə");
            ipamap.put("ua", "ʊə");
            ipamap.put("hh", "h");
            ipamap.put("p", "p");
            ipamap.put("b", "b");
            ipamap.put("t", "t");
            ipamap.put("d", "d");
            ipamap.put("k", "k");
            ipamap.put("m", "m");
            ipamap.put("n", "n");
            ipamap.put("l", "l");
            ipamap.put("r", "ɹ");
            ipamap.put("f", "f");
            ipamap.put("v", "v");
            ipamap.put("s", "s");
            ipamap.put("z", "z");
            ipamap.put("w", "w");
            ipamap.put("g", "ɡ");
            ipamap.put("y", "j");

            List<String> lines = FileUtils.readLines(new File("beep/beep-1.0"), "UTF-8");
            for (String l : lines) {
                String left  = StringUtils.substringBefore(l, "\t");
                String right = StringUtils.substringAfterLast(l, "\t");
                for (Entry<String, String> e : ipamap.entrySet()) {
                    right = right.replace(e.getKey(), e.getValue());
                }
                right = right.replace(" ", "");
                FileUtils.writeStringToFile(new File("beep-1.0-ipa"),
                                            String.format("%s\t%s\n", left, right),
                                            "UTF-8",
                                            true);
            }
        }

        {
            Map<String, String> ipamap = new LinkedHashMap<>();
            ipamap.put("ɛɪ", "ei");
            ipamap.put("aɪ", "ai");
            ipamap.put("ɔɪ", "òi");
            ipamap.put("oʊ", "où");
            ipamap.put("aʊ", "aù");
            ipamap.put("ɪə", "iø");
            ipamap.put("ɛə", "eø");
            ipamap.put("ʊə", "ùø");

            ipamap.put("p", "p");
            ipamap.put("b", "b");
            ipamap.put("t", "t");
            ipamap.put("d", "d");
            ipamap.put("k", "k");
            ipamap.put("m", "m");
            ipamap.put("n", "n");
            ipamap.put("l", "l");
            ipamap.put("ɹ", "r");
            ipamap.put("f", "f");
            ipamap.put("v", "v");
            ipamap.put("s", "s");
            ipamap.put("z", "z");
            ipamap.put("h", "h");
            ipamap.put("w", "w");
            ipamap.put("ɡ", "g");
            ipamap.put("tʃ", "ĉ");
            ipamap.put("dʒ", "ĝ");
            ipamap.put("ŋ", "ǹ");
            ipamap.put("θ", "ŧ");
            ipamap.put("ð", "đ");
            ipamap.put("ʃ", "ŝ");
            ipamap.put("ʒ", "ĵ");
            ipamap.put("j", "j");
            ipamap.put("iː", "<ħ>ï</ħ>");
            ipamap.put("ɑː", "<ħ>ā</ħ>");
            ipamap.put("ɔː", "<ħ>ò</ħ>");
            ipamap.put("uː", "<ħ>u</ħ>");
            ipamap.put("ɜː", "<ħ>ȑ</ħ>");
            ipamap.put("ɪ", "i");
            ipamap.put("ɛ", "é");
            ipamap.put("æ", "ä");
            ipamap.put("ʌ", "á");
            ipamap.put("ɒ", "o");
            ipamap.put("ʊ", "ù");
            ipamap.put("ə", "ø");

            ipamap.put("ts", "c");
            List<String> lines = FileUtils.readLines(new File("beep-1.0-ipa"), "UTF-8");
            for (String l : lines) {
                String left  = StringUtils.substringBefore(l, "\t");
                String right = StringUtils.substringAfterLast(l, "\t");
                for (Entry<String, String> e : ipamap.entrySet()) {
                    right = right.replace(e.getKey(), e.getValue());
                }
                FileUtils.writeStringToFile(new File("beep-1.0-ipa-ergo"),
                                            String.format("%s\t%s\n", left, right),
                                            "UTF-8",
                                            true);
            }
        }
    }
}