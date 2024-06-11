package org.kinode;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URI;
import java.util.UUID;

public class Mojang {
  private static final String MOJANG_API_URL = "https://api.mojang.com/user/profiles/";

  public static String getPlayerId(UUID uuid) {
    try {
      // Construct the URL
      URI uri = new URI(MOJANG_API_URL + uuid.toString().replace("-", "") + "/names");
      HttpURLConnection connection = (HttpURLConnection) uri.toURL().openConnection();
      connection.setRequestMethod("GET");

      // Read the response
      BufferedReader in = new BufferedReader(new InputStreamReader(connection.getInputStream()));
      String inputLine;
      StringBuilder content = new StringBuilder();
      while ((inputLine = in.readLine()) != null) {
        content.append(inputLine);
      }

      // Close connections
      in.close();
      connection.disconnect();

      // Parse the JSON response to get the latest username
      String jsonResponse = content.toString();
      String[] names = jsonResponse.split(",");
      String latestName = names[names.length - 1];
      latestName = latestName.split(":")[1].replace("\"", "").replace("}", "");

      return latestName;
    } catch (Exception e) {
      e.printStackTrace();
      return null;
    }
  }
}
