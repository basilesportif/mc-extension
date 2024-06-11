package org.kinode;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URI;
import java.util.UUID;
import org.json.JSONObject;

public class MojangAPI {
  private static final String MOJANG_API_URL = "https://api.mojang.com";

  public static String getPlayerId(UUID uuid) {
    try {
      // Construct the URL
      String userProfile = "/user/profile/";
      URI uri = new URI(MOJANG_API_URL + userProfile + uuid.toString().replace("-", ""));

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
      JSONObject jsonObject = new JSONObject(jsonResponse);
      String playerName = jsonObject.getString("name");

      return playerName;
    } catch (Exception e) {
      e.printStackTrace();
      return null;
    }
  }
}
