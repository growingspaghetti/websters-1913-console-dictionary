import java.io.File;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

import org.apache.commons.io.FileUtils;
import org.apache.commons.lang3.StringUtils;

// https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap
public class WordNetLineBreaker {
  private static final List<String> ONE_TO_NINE_ONE_SPACE =
      IntStream.range(1, 10).mapToObj(i -> " " + i + " ").collect(Collectors.toList());
  private static final List<String> ONE_TO_NINE_TWO_SPACES =
      IntStream.range(1, 10).mapToObj(i -> "  " + i + " ").collect(Collectors.toList());
  private static final List<String> TEN_TO_HUNDRED_ONE_SPACE =
      IntStream.range(10, 100).mapToObj(i -> " " + i + " ").collect(Collectors.toList());

  public static void main(String args[]) throws Exception {
    List<String> lines =
        FileUtils.readLines(new File("wordnet-ipa.txt"), StandardCharsets.UTF_8);

    List<String> results = new ArrayList<>();
    for (String line : lines) {
      String contents = StringUtils.substringAfter(line, "\t");
      String title = StringUtils.substringBefore(line, "\t");

      String[] linesInEntry = contents.split("\\\\n");
      List<String> entry = new ArrayList<>();
      Arrays.asList(linesInEntry).forEach(l -> entry.addAll(applyLineWrap(l)));
      results.add(title + "\t" + entry.stream().collect(Collectors.joining("\\n")));
    }
    FileUtils.writeLines(new File("wordnet-ipa-line-break-80.txt"), results);
  }

  private static List<String> applyLineWrap(String lineInEntry) {
    lineInEntry = singleDigitIndent(lineInEntry);
    String indent = getIndentation(lineInEntry);

    lineInEntry = StringUtils.stripEnd(lineInEntry, null);
    String[] words = StringUtils.splitPreserveAllTokens(lineInEntry, " ");
    StringBuilder sb = new StringBuilder();
    List<String> sublines = new ArrayList<>();
    for (String w : words) {
      if ((sb.toString() + w).length() < 80) {
        sb.append(w + " ");
      } else {
        sublines.add(StringUtils.stripEnd(sb.toString(), null));
        sb.delete(0, sb.length());
        sb.append(indent);
        if (!StringUtils.isEmpty(w)) {
          sb.append(w + " ");
        }
      }
    }
    if (!StringUtils.isEmpty(sb.toString())) {
      sublines.add(StringUtils.stripEnd(sb.toString(), null));
    }
    return sublines;
  }

  public static String singleDigitIndent(String l) {
    if (ONE_TO_NINE_ONE_SPACE.parallelStream().anyMatch(l::startsWith) || l.startsWith("   [")) {
      return " " + l;
    }
    return l;
  }

  private static final String FOUR_SPACES = "    ";
  private static final String FIVE_SPACES = "     ";
  private static final String SIX_SPACES = "      ";

  public static String getIndentation(String l) {
    if (l.startsWith("n : ")) {
      return FOUR_SPACES;
    } else if (l.startsWith("v : ")) {
      return FOUR_SPACES;
    } else if (ONE_TO_NINE_TWO_SPACES.parallelStream().anyMatch(l::startsWith)) {
      return FOUR_SPACES;
    } else if (TEN_TO_HUNDRED_ONE_SPACE.parallelStream().anyMatch(l::startsWith)) {
      return FOUR_SPACES;
    } else if (l.startsWith("    [")) {
      return FIVE_SPACES;
    } else if (l.startsWith("adj : ")) {
      return SIX_SPACES;
    } else if (l.startsWith("adv : ")) {
      return SIX_SPACES;
    } else if (l.startsWith(" ")) {
      throw new IllegalArgumentException(l);
    } else {
      return "";
    }
  }
}
