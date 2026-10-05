import 'package:flutter/material.dart';

Color highlightColor(String color) => switch (color) {
      'green' => Colors.green.shade300,
      'blue' => Colors.blue.shade300,
      'pink' => Colors.pink.shade200,
      'purple' => Colors.purple.shade200,
      _ => Colors.amber.shade300,
    };

String highlightCssColor(String color) => switch (color) {
      'green' => '#a5d6a7',
      'blue' => '#90caf9',
      'pink' => '#f48fb1',
      'purple' => '#ce93d8',
      _ => '#ffe082',
    };
